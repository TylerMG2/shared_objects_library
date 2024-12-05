Ideally we want the library to provide secure updates whilst still being easy for the client to use.

The issue at the moment is we would be allowing the client to update any field in the struct and send it over, that would then require anyone using the library to check every
value for changes that aren't supposed to be their for a given event before allowing it through.

There are a few solutions I can think of for this:
1. Create a proc macro for a events and have the user of the library specify the related fields for each client event (we don't care about server events)
- Issues: Arrays.
2. Add ownership to structs
- Issues: Gets pretty complex also can still update alot of unnecessary fields
3. Don't have the client update fields directly, instead have them send all the required information in the events (like my first implementation). This works fine since
	the server already has to implement a Handle Client event function so no extra work for the consumer. This method also makes validating moves much easier.
	The only major downside is the consumer will have to specify how the room state is updated (as opposed to client) but the other benefits far outweigh this and for 
	events like GameStart where it has to deal cards the server would have to do extra work anyway.
