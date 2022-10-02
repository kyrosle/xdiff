# xdiff
the rust learning project

useage:

- xdiff 
- xreq

## xdiff
compare two `url api` and able to skip some `header` or `body`

### for interaction cli
run command : 
```
xdiff parse
```
### for cli with yaml config

the example is in the code (fixtures/test.yaml)

run command : 
```
xdiff run -p (yaml config node name) -c (yaml config file path) -e(*) some param(s)
```


## xreq
just like the cli tool `curl` but you are able to use yaml config

### for interaction cli
run command : 
```
xreq parse
```
### for cli with yaml config

the example is in the code (fixtures/xreq_test.yaml)

run command : 
```
xreq run -p (yaml config node name) -c (yaml config file path) -e(*) some param(s)
```


just for learning to write a cli project

- how to thinking the data struct
- how to thinking the rebuild action for whole code
- how to make good use of unit test

with using tool

- cargo install nextest --locked

the initial project from : https://github.com/Tubitv/xdiff
