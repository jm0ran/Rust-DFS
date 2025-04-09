# Rust Distributed File Sharing
## Pre-Shared File
This program is designed so that at the minimum a client must have the SHA3-512 file hash of the file they are requesting. Additionally, the program should have the address of a linking server to query for clients who are distributing their requested files and to broadcast the files they are distributing.

## Protocol
### Discovery (Client -> Linking Server)
Client to linking server requests occur when the client connects to a new linking server or is requesting an update either looking for new distributors or requesting new files. The request is sent as a collection of lines with # annotations used to split the request body into parts. A request has the following form:
```
#S RDFS 0.1 DISCOVERY_POST 16000
#A 127.0.0.1:7779
#D
[Hashes of Distributing Files, 1/line]
2dd184b8c84b999a6ccc7ae4da2efc3b3cd455d50a04686caaf90f8f5cd60194c8e0e758947738f1001e01010ddb28e782ed274c966561ba798fe0123f495b5d
#R
[Hashes of Requesting Files, 1/line]
c38b125b5e5ad6ce29a9d9b18b768e74d66f4ea71a2a3be8f8dd74078332965110f0b3b3052f3d7612f556745e8ad26dedd92389bf4d236d25ef4d8f2924e054
#E
```
The first line of the request sends some initial client information #S represents the start of a new request then we include the application, the version, the operation we are performing on the linking server, and then very importantly the block size of data being distributed (it is critical that connected clients share a common block size, for this reason we set it on the linking server level so that all clients that want to connect must conform). On the next line #A represents the broadcast address of the client. It is important to note that this ip:port combo is not what the linking server is connected to but is instead the target that other clients would use to connect to the sending client. Next, #D represents distributing and indicates to the server that the next lines up to #R will all contain a hash for each file the client is distributing. These are currently implemented as SHA3-512 hashes to minimize possible collisions but this may change in the future. After all the distributing hashes we reach #R which represents the start of the requesting hashes which are also SHA3-512. We read the requesting hashes until we reach #E which represents the end of the request.

Moving to the response, we will be informing the client of the distributors for the files that they are requesting. The response constructed by the linking server will have the following form:

```
#S RDFS 0.1 DISCOVERY_POST 16000
#T
[HASH OF REQUESTED FILE] [DISTRIBUTOR 1] [DISTRIBUTOR 2] …
2dd184b8c84b999a6ccc7ae4da2efc3b3cd455d50a04686caaf90f8f5cd60194c8e0e758947738f1001e01010ddb28e782ed274c966561ba798fe0123f495b5d 127.0.0.1:8072 127.0.0.3:8072
#E
```