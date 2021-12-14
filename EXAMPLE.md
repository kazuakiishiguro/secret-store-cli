# integration example

## workflows

if you haven't finished building `secret-store-cli` file, please follow [README.md](README.md) in install section.

### encryption

1 : prepare your document that you want to encrypt. (e.g: main.js)
```bash
$ cat main.js

console.log("Hello World!");
```

2 : encrypt `main.js` document. (make sure your secret store node is running, otherwise it will fail.)

* type password for eth account.
* then you will get `dockey_id` and `ipfs_hash` for your document.

```bash
$ ./target/release/secret-store-cli encrypt main.js
Please note that password is NOT RECOVERABLE.
Type password:
Repeat password:
dockey_id: 0x3f313565369cdde60e7d4b85622e6bb4edaa69e86fc1d43183f9c56ad9584de9
ipfsHash: "QmNfnryjxcGQJQfJRoaFv7D1kCRWirgUFMW7YCXjTZkXi4"
```

secret store node response

```bash
ss2_1   | 2019-09-19 03:45:55 UTC 0xda87…0495: generation session completed
ss3_1   | 2019-09-19 03:45:55 UTC 0xaea5…5c36: generation session completed
ss1_1   | 2019-09-19 03:45:55 UTC 0xdbb5…705e: generation session completed
ss3_1   | 2019-09-19 03:46:01 UTC 0xaea5…5c36: encryption session completed
ss2_1   | 2019-09-19 03:46:01 UTC 0xda87…0495: encryption session completed
ss1_1   | 2019-09-19 03:46:01 UTC 0xdbb5…705e: encryption session completed
```

encrypted document `3f313565369cdde60e7d4b85622e6bb4edaa69e86fc1d43183f9c56ad9584de9` will look something like this :

```3f313565369cdde60e7d4b85622e6bb4edaa69e86fc1d43183f9c56ad9584de9
"0xb6fa6a9a9e25731170bab15b5a6d9d5275d9c3e6b619fd00eac238ee9d2010ef65489896c6d06b19bedc3bc90a952133c533f592d233d59a"
```

also you can check ipfs_uri [here](https://ipfs.infura.io/ipfs/QmNfnryjxcGQJQfJRoaFv7D1kCRWirgUFMW7YCXjTZkXi4).

3 : decrypt with `dockey_id` and `ipfs_hash`

here is a very simple version of how to read encrypted document from unity's `C#` script. you need to do followings before running decryption.

* move `secret-store-cli` file to your root directory of unity project.
* hard-code `dockey_id` and `ipfs_uri` to `ProcessStartInfo`'s arguments.
* hard-code account password.

```C#
using UnityEngine;
using System;
using System.IO;
using System.Text;
using System.Diagnostics;

public class Decrypt : MonoBehaviour
{
    void Start()
    {
        ProcessStartInfo psi = new ProcessStartInfo("secret-store-cli "); // move to your root directory.
        psi.Arguments = "decrypt 3f313565369cdde60e7d4b85622e6bb4edaa69e86fc1d43183f9c56ad9584de9 QmNfnryjxcGQJQfJRoaFv7D1kCRWirgUFMW7YCXjTZkXi4"; // update your own. TODO : deal with this in the future with web3 proxy.
        psi.UseShellExecute = false;
        psi.RedirectStandardInput = true;
        psi.RedirectStandardOutput = true;

        using (Process child = Process.Start(psi))
        {
            child.StandardInput.WriteLine("5"); // TODO : work something for betther user experience
            child.StandardInput.WriteLine("5");
            child.StandardInput.Close();

            child.WaitForExit();
            string stdout = child.StandardOutput.ReadToEnd();
            UnityEngine.Debug.Log(stdout);
        }
    }
}
```

4 : read decrypted document from `stdout`. in your unity's debug log, you will see the followings.

```unity's debug
console.log("Hello World!");
```

ss-node response
```
ss3_1   | 2019-09-19 04:08:23 UTC 0xaea5…5c36: version negotiation session completed
ss1_1   | 2019-09-19 04:08:23 UTC 0xdbb5…705e: version negotiation session completed
ss2_1   | 2019-09-19 04:08:23 UTC 0xda87…0495: version negotiation session completed
ss1_1   | 2019-09-19 04:08:23 UTC 0xdbb5…705e: version negotiation session read error 'no active session with given id' when requested for session from node 0xda87…0495
ss2_1   | 2019-09-19 04:08:23 UTC 0xda87…0495: version negotiation session read error 'no active session with given id' when requested for session from node 0xdbb5…705e
ss1_1   | 2019-09-19 04:08:23 UTC 0xdbb5…705e: decryption session completed
ss3_1   | 2019-09-19 04:08:23 UTC 0xaea5…5c36: decryption session completed
ss2_1   | 2019-09-19 04:08:23 UTC 0xda87…0495: decryption session completed
```