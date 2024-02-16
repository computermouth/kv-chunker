# kv-chunker

A fastly compute app which stitches together bodies of chunked artifacts for bulk hosted files in KVStore.

## how does it work

A given file is chunked into 25MiB segments with the `uploader.sh` script, and is `PUT` in the KVStore along with a metadata file which just states how many chunks exist. Example:

```
$ ls -lsah myfile
48M myfile
$ stat --printf="%s" myfile 
50000000
$ ./uploader.sh myfile ${PASSWORD} ## PASSWORD must be set in src/pw.rs
```

With this particular file, 3 files will be inserted in your fastly KVStore
```
└── mykvstore
    ├── myfile_pcs
    ├── myfile_0
    └── myfile_1
```

`myfile_pcs` contains a printable text number of the chunks. In this case: 2.
`myfile_0` is the first 25MiB(26214400B) of `myfile`
`myfile_1` is the remaining ~23MiB(23785600B) of `myfile`

Larger files will be `n` number of chunks, with the final chunk always being `${FILE_SIZE_IN_BYTES} % 26214400` bytes long.

On GETs, `myfile_pcs` is fetched, and it's contents parsed. The compute app will create N number of lookups, and append the bodies to the outbound response body.

`TODO` -- use lookup_async, and gather a vector of handles, before waiting on all of them. (Currently a bug preventing this where PendingInsertHandle is not exported correctly).

## getting started

Create a file `src/pw.rs`, and add the following

```
pub const PASSWORD: &str = "mysupersecretpassword";
```

`DISCLAIMER`
This password gets embedded in the application. As there's no rate limiting involved in the application, this is incredibly susceptible to a brute force attack. If used in any production scenario, at a minimum, please please please disable the `put()` logic once you're done with your uploads, or better yet, devise an actual authentication mechanism.

## recommended usage

0. (create fastly account, create a kv store)
1. set `PASSWORD``, enable the `PUT` path in `main.rs`
2a. deploy
2b. (link your kv store with the compute service)
3. use uploader `$ ./uploader.sh myfile ${PASSWORD}`
4. test your GET -- `curl -XGET https://your-app-subdomain.edgecompute.app/myfile -O`
5. disable `PUT` path
6. deploy