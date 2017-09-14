    sudo tcpdump -U -n -i any | grep --line-buffered '> 192.168.1.230' | cargo run
