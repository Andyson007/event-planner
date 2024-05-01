# Event-planner
This pretty much just allows for a quick and dirty mock up for planning together with your friends on discord. I have not spent too much effort on it yet, but if you have any ideas for additions please suggest them.

## Setup
Create a .env file in the root folder (src/..) with the discord token like ```
DISCORD_TOKEN=TOKEN_HERE
```
then just do cargo run and it should work
(If this doesn't tell me, I have not spent much effort on this)

### Timezones
It uses the system timezone for now so you will have to set it to match your local time

## Knows limitations
I haven't bothered implementing special cases for times when we swap between summertime and wintertime because It's a massive pain, with little to no payback. If you are using this bot, just don't plan events for ~3 AM during this swap :)

## Plans for the future
-[ ] Implement better time parsing
