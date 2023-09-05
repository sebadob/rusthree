# rusthree

This is a fork from [rust-s3](https://crates.io/crates/rust-s3) which has quite a few problems imho.  
This fork is just there to get rid of all unnecessary stuff and only provide [tokio](https://crates.io/crates/tokio)
compatibility.

The original crate tries to combine sync and asnyc into one crate and this created a lot of problems. In many places,
the original crate uses blocking calls even with only async features enabled. If you just need to put a file from
time to time, that might be fine, but this becomes a real problem if you are using S3 / Minio for things like static
file serving (like I do) or if you want to just "do a lot" on S3 storage from your application.

Additionally, the original crate just uses low level [hyper](https://crates.io/crates/hyper) under the hood. This is
fine for an access once in a while, but becomes a real problem in high throughput scenarios. This basically creates 
a completely new TLS connection for each single request and the TLS handshakes take an enormous portion of your
resources.  
The new utilization of reqwest's connection pooling makes the storage in my case even usable for static file and asset
serving, when you proxy the data through your API, which you want to do if you need restricted access for instance.

This crate got rid of most of the stuff which is unnecessary for me and changed from low level hyper to using 
[reqwest](https://crates.io/crates/reqwest) to take advantage of the internal connection pool. The reduced amount of TLS
handshakes is just out of this world if you have a lot of accesses.

Another really big problem I had with the original crate was the `put_object_stream`. This took a stream and you might
think it is just streaming the file as you read it, but it actually reads the whole file into memory first before it
does any send to the storage. If you are handling huge files, you can get OOM killed by the Linux kernel really quickly.
This was changes as well, and it uploads a chunk as soon as it was read.

Mutex'es and locking structures were used in the original crate during each request as well, even when it would not
even be needed.
This crate is only working with access key and secret only for now and not with dynamic sessions.

## IMPORTANT

This crate is currently in the state of "just make it work somehow" and NOT production ready if you "just want to use it".

A lot of the original test cases were just deleted (for now) or commented out for now to just get it working again.
There are a lot of things I want to change about this fork. Rename stuff, re-organize crates and modules. Additionally,
there is a lot of room left for optimization. I just did the most important things for my use case for now.

It does not make sense to open any issues for now, since there is just a lot of work to be done anyway.