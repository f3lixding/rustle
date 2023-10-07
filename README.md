# Rustle
A PiHole implemented in Rust.
(Or at least what I imagine PiHole to be).

On a high level, this is a DNS server that does the following:
- Receives DNS queries (via UDP).
- Decodes DNS queries.
- Runs the queried domains through a rule engine (supplied by easy list).
- If a given queried domain matches a rule, it gives a response that prompts a noop by the client (i.e. 0.0.0.0).
- If a given queried domain does not match any rule, it then tries delegate the look up of it to another DNS server (i.e. the router).
- Returns the appropriate response to the client (i.e. encoded in the expected format).

## Relevant Stuff
In the process of implementing this project, I have come across the following DNS related learnings that I find relevant (to be updated):
- [How to decode/encode a DNS query/response](https://cabulous.medium.com/dns-message-how-to-read-query-and-response-message-cfebcb4fe817)
- [How to find the router's addresss](https://crates.io/crates/ipconfig)
- [Easylist as provided by easylist io](https://easylist.to/)
