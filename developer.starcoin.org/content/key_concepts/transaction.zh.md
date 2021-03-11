---
title: 交易
weight: 2
---

Starcoin区块链的用户通过客户端提交签名后的交易，更新链上的账本状态。

<!--more-->

签名交易主要包括以下内容：

- **发送地址** —— 交易发送者的账户地址
- **发送公钥** —— 由发送者签署交易的私钥生成的公钥
- **程序** —— 程序可能包含以下部分：
  - Move脚本的字节码
  - 输入参数。对于点对点交易，输入包括接收者的信息和金额
  - 要发布的Move模块的字节码
- **序列号** —— 一个无符号整数，必须等于发送者账户下存储的序列号
- **过期时间** —— 交易失效的时间
- **签名** —— 发送者的数字签名