# Refactoring
- doc function doc function doc function doc function doc function

## done.
- Properly throw and propagate error
- make stream plugable to facilitate testing
- store detailed information in the error objects and generate a proper error message
- done?: Figure out all the ownership stuff in slices and vecs for the parameters when writing functions
- test client with mock stream

    - differenciate packets when reading from the stream by only reading the packet size amount

# Stability
- use unique id for each request
- deal with server responding with two packets(empty resp, and then authresp) when responding to auth.
    - Make the client wait for the resp with the right id, storing resps irrelevant to the req in the struct and throwing out old irrelevant resps.
    This thing is weird. Minecraft is not following the doc and only sending the authresp

# Functionality
- ~~Verbose command line option to facilitate debuging~~ for a library???
- implement Read Write Unpin trait to make it like a stream object??