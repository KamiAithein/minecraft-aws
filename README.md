# minecraft-aws
## What is it?
A heroku deployable server written in rust with rocket that communicates with AWS EC2 via rusoto allowing one to control a server located on an EC2 instance remotely.
I have used this solution to reduce costs of running a server by having the server turn off when no one is on, and allowing anyone to turn on the server 
by accessing the Heroku deployed interface.

## Note
This was attempted while I was learning rust and is not actively maintained (it works for now so I haven't had to tweak it) but it was also created while some features I wanted to use were not in standard rust, so it is a little bit wonky. The main rust file is also close to illegible. This is more for reference than for actual use as a product (aws interactions are split between src/aws and src/mc_server)
