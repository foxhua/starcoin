use crate::helper;
use actix::prelude::*;
use anyhow::{bail, Result};
use bus::{Bus, BusActor};
use logger::prelude::*;
use network::NetworkAsyncService;
use starcoin_sync_api::sync_messages::{GetTxns, StartSyncTxnEvent};
use starcoin_txpool_api::TxPoolAsyncService;
use txpool::TxPoolRef;
use types::peer_info::PeerId;

#[derive(Clone)]
pub struct TxnSyncActor {
    bus: Addr<BusActor>,
    inner: Inner,
}

impl TxnSyncActor {
    pub fn launch(
        txpool: TxPoolRef,
        network: NetworkAsyncService,
        bus: Addr<BusActor>,
    ) -> Addr<TxnSyncActor> {
        let actor = TxnSyncActor {
            inner: Inner {
                pool: txpool,
                network_service: network,
            },
            bus,
        };
        actor.start()
    }
}

impl actix::Actor for TxnSyncActor {
    type Context = actix::Context<Self>;

    /// when start, subscribe StartSyncTxnEvent.
    fn started(&mut self, ctx: &mut Self::Context) {
        let myself = ctx.address().recipient::<StartSyncTxnEvent>();
        self.bus
            .clone()
            .subscribe(myself)
            .into_actor(self)
            .map(|res, _act, ctx| {
                if let Err(e) = res {
                    error!("fail to subscribe start_sync_txn event, err: {:?}", e);
                    ctx.terminate();
                }
            })
            .wait(ctx);

        info!("Network actor started ",);
    }
}

impl actix::Handler<StartSyncTxnEvent> for TxnSyncActor {
    type Result = ();

    fn handle(
        &mut self,
        _msg: StartSyncTxnEvent,
        ctx: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        self.inner
            .clone()
            .sync_txn()
            .into_actor(self)
            .map(|res, _act, _ctx| {
                if let Err(e) = res {
                    error!("handle sync txn event fail: {:?}", e);
                }
            })
            .spawn(ctx);
    }
}

#[derive(Clone)]
struct Inner {
    pool: TxPoolRef,
    network_service: NetworkAsyncService,
}

impl Inner {
    async fn sync_txn(self) -> Result<()> {
        // get all peers and sort by difficulty, try peer with max difficulty.
        let mut best_peer = self.network_service.peer_set().await?;
        best_peer.sort_by_key(|p| p.total_difficult);
        best_peer.reverse();

        for peer in best_peer {
            match self.sync_txn_from_peer(peer.peer_id).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    error!("fail to sync txn from peer, e: {:?}", e);
                }
            }
        }

        bail!("fail to sync txn from all peers")
    }
    async fn sync_txn_from_peer(&self, peer_id: PeerId) -> Result<()> {
        let txn_data = helper::get_txns(&self.network_service, peer_id.clone(), GetTxns)
            .await?
            .txns;
        let import_result = self.pool.clone().add_txns(txn_data).await?;
        let succ_num = import_result.iter().filter(|r| r.is_ok()).count();
        info!("succ to sync {} txn from peer {}", succ_num, peer_id);
        Ok(())
    }
}
