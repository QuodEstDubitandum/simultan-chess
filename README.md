# About 

An implementation of chess in Rust. The actix web-server allows to play a game of chess by interacting with its REST API. The information about the game is saved 
in a local SQLite-like file using TursoDB.

I use this backend service on my personal website, so its tailored to my use case, which would be: 
- A user makes a make which is validated at `/game/validate`
- If its valid, the user is asked to confirm their move (in case they fatfingered). Once he confirms, `/game/vote` will add the vote for the specific move to the DB.
- Everyday at midnight, the move with the most votes get played using `/game/move`

To display additional information in the frontend, we also have some routes for fetching the history and the current game state as well as the possibility 
to finish a game manually just in case (after performing a move, we check whether game is finished automatically).
