# Clipable

## Testing

```Batch
cargo build
dev_install.bat
```

## Notes

make a global channel, listener thread will use global channel to stop
listener thread watching the folder for changes
stopper and reloader need to be able to kill the listener thread 
https://stackoverflow.com/questions/26199926/how-to-terminate-or-suspend-a-rust-thread-from-another-thread
look at passcod/notify for folder listening
need to store streamable credentials somewhere, probably registry
could also be service params
when a new .mp4 that's < 1gb hits the folder, upload to streamable
save streamable URL in a txt file with the .mp4 file name

Create("C:\\Users\\rudes\\Desktop\\Tom Clancy\'s Rainbow Six  Siege 2019.01.09 - 13.00.20.26.DVR_Trim12.mp4")