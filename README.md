# 支持的操作

* `MpcPeer::NewSession` 创建分布式会话
* `MpcPeer::Keygen` 创建门限多签密钥 (以下简称密钥)
* `MpcPeer::KeygenMnem` 导入助记词创建密钥
* `MpcPeer::Sign` 门限多签
* `MpcPeer::Reshare` 从已有的密钥创建新的密钥, 新老密钥具有相同的私钥, 但成员结构不同.

以上所有接口都是阻塞的.

# 编译和部署

(1) 进入项目根目录, 执行命令 `make` . 执行成功后, 项目 `out/` 目录下将出现 `svarog_peer`, `svarog_sesman` 等文件.

(2) 运行 `svarog_peer`, `svarog_sesman` 这两个程序.

> 这两个程序无需命令行参数就能运行. 用户也可以自行探索它们的命令行参数, 以修改它们监听的端口和 ip .

# MpcPeer::NewSession

一场会话由元组 `(sesman_url, session_id)` 唯一确定. 其中,

* `sesman_url` 是 `svarog_sesman` 服务的 URL; 
例如, 在 `example.org:9000` 部署 `svarog_sesman`, 那么 `sesman_url` 就是 `http://example.org:9000` .
* `session_id` 既可以由用户指定, 也可以交给 sesman 来随机生成.
由 sesman 生成的 `session_id` 是去掉连字符的小写 UUID-v4 .

创建会话之后才能开展 `Keygen`, `KeygenMnem`, `Sign`, `Reshare` 操作.
开展这些操作, 需要用不同的方式来填写 `SessionConfig`. 将在各操作的说明里介绍填写方式.

> 通过将 `SessionConfig.session_id` 字段设为 **空字符串**, 就可以让 sesman 随机生成 session_id .

# MpcPeer::Keygen

(1) 收集 `players` 名单, 以及门限 `threshold` .
`players` 是一个 `map<String, bool>`, 键为参与方的名称, 值为是否出席. 对 Keygen 会话来说, 值一律为 `true`.

(2) 填写并提交 `SessionConfig`. 
必填字段: `algorithm, sesman_url, threshold, players` . 

(3) 各参与方填写并提交 `ParamsKeygen`. 接口返回 `Keystore`; 各参与方需妥善保存它, 避免丢失和泄露.

# MpcPeer::KeygenMnem

(1) 收集 `players` 名单, 以及门限 `threshold` . 收集方式与 Keygen 相同.

(2) 填写并提交 `SessionConfig`. 填写方式与 Keygen 相同.

(3) 各参与方填写并提交 `ParamsKeygenMnem`. 注意: 恰有一方提供助记词. 接口返回 `OptionalKeystore`, 意思是拆包后有可能得到 `Keystore`, 也有可能是空的.

> 如果助记词提供者不持有分片; 也就是 `member_name` 填 **空字符串**, 同时还提供助记词; 那么有且只有助记词提供者所得到的 `OptionalKeystore` 拆出来是空的. 其他情况都能拆出来 `Keystore` .

# MpcPeer::Sign

(1) 收集 `players` 名单. 需要注意:
* `players` 的键必须曾经参与同一场 Keygen , 或 KeygenMnem , 或作为 Reshare consumer; 必须恰好包含前述人员, 不能增删改.
* `players` 的值决定了是否出席本场会话. 出席人数应不小于相应的门限, 否则签名将失败.

(2) 填写和提交 `SessionConfig`. 必填字段: `algorithm, sesman_url, players`.

(3) 各参与方填写并提交 `ParamsSign`. 接口返回 `Signature`.

# MpcPeer::Reshare

(1) 收集 `players` 名单. 收集方式与 Sign 相同. 这些人是 Reshare provider .

(2) 收集 `players_reshared` 名单, 以及门限 `threshold` . 收集方式与 Keygen 和 KeygenMnem 相同. 这些人是 Reshare consumer .

> 这两个名单允许有交集.

(3) 填写并提交 `SessionConfig`. 
必填字段: `algorithm, sesman_url, threshold, players, players_reshared` . 注意 `threshold` 与 `players_reshared` 而不是 `players` 对应.

(4) 各参与方填写并提交 `ParamsReshare`. 接口返回 `OptionalKeystore`; 如果参与方不是 consumer, 那么一定拆出空; 如果参与方是 consumer, 那么一定拆出 `Keystore`.