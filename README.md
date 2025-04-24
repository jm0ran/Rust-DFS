# Rust Distributed File Sharing 
## [Demo Video](https://www.youtube.com/watch?v=K2Y0zqv0KnY)

## Pre-Shared File
This program is designed to accept a shared file that has the following information. The SHA3-512 hash of the requested file. The block size that the file will be downloaded in, this block size should be shared between clients and the linking server. The file should then contain the hashes of each block. The final block is unique as it may not be full, as a result the associated line will also include the size of the block. The pre-shared file is essentially a text file and will take the following form:

```
#S [Block Size] [File Siz] [Requested File Hash]
[Complete Block Hash 1]
[Complete Block Hash 2]
[...]
#E [Final Block Size] [Final Block Hash]
```

## Client Commands
### General
**Exit**: Exit the client application and halt any current downloads, will consider a clean exit option that waits for all currently processing blocks to conclude
```
exit
```
**Clear**: Clears the current terminal screen, just wanted to add this because I like it for debugging
```
clear
```
### Linker
**Set**: Set the linking server address for the client to use. Address should be in form of `IP:Port`.
```
linker set 127.0.0.1:8080
```
**Get**: Get the currently set address of the linker, prints this address to standard output
```
linker get
```
**Update**: Instruct linker object to request new data from the linking server. While this should be run whenever new requests and distributing files are registered as well as on a constant interval, this command lets the client run the operation manually.
```
linker update
```
### File
**Scan**: Scan the distributing directory to load files and their corresponding hashes into the file manager. Currently only performs a shallow search on the directory. This directory can be configured in the config.rs file
```
files scan
```
**Distributing**: Get distributing file paths as well as their corresponding hashes.
```
files distributing
```

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
The first line is the same as the initial request and provides information on the protocol version being used as well as the corresponding action. On the second line, #T represents the start of the targets on a file by file basis. Each next line up tp #E has the hash of the requested file as well as a collection of distributors who have that file available. #E indicates that the request is complete and all distributors have been read. As an important note all requested files should be returned here, if a file has no distributors then the line should simply contain the file hash.

### Data Request (Client -> Client)
Once clients have acquired a target for their requested file they can make a request to it. A request focuses on a single block of data based on the previously defined data size. The request from the receiving to the distributing client takes the following form:

```
#S RDFS 0.1 BLOCK_REQUEST 1600
#T 2dd184b8c84b999a6ccc7ae4da2efc3b3cd455d50a04686caaf90f8f5cd60194c8e0e758947738f1001e01010ddb28e782ed274c966561ba798fe0123f495b5d
#B 122
#E
```
The first line is identical to our other requests the only difference being the BLOCK_REQUEST tag denoting that this message is requesting a block of data. Next we have #T which denotes that this line contains the target file hash that this client is looking for. Then we have #B which denotes the numbered block this client is looking for. Finally we have #E which denotes the end of this request.

Looking at our response, if the request is successful, it will take the following form:
```
#S RDFS 0.1 BLOCK_REQUEST 1600
#D [Data Size (useful for final block)]
[RAW BITES OF BLOCK SIZE]
#E
```
Note that this is pretty straightforward. We have an identical header and then we have a #D to tell the client that data will follow. The client will read the corresponding block size and then the #E denotes the conclusion of the response.