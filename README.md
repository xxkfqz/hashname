# hashname
Utility for renaming files to their SHA256 hash. It is mainly used to find and delete identical files within the same directory.

## Examples

```shell
$ hashname ./data0.bin ./data1.bin ./data2.bin ./data3.bin
$ ls
0ac074949ad046dbd12d5be323cb0a9bcc9bb913d6f76defa57443a8c0eb64fc.bin
dd01352a1c9ba7be8343bc149a8a9f70529349f6147c73648a6ca4941e124f6e.bin
aeda128839e71ff642eadbeb6dff76829a80284986249840c4cf4235c47fa8ba.bin
f18a23f874703d435516bdc6267d29ce93d7c2be48a2aaa1a9620600a99f0ae9.bin
```