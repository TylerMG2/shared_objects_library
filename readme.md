## Networked Trait
The network derive macro will both implement the Networked trait but it will also create an optional version of the struct. It will support primitves, arrays and other types that also derive the Networked trait. For non primitive types that implement the Networked trait, it will replace the type with an Option<Vec<u8>> Vec<u8> being the output of bincode::serialize. For example

struct Room {
	players: [Option<Player>; 8]
}

struct _OptionalRoom {
	players: Option<[Option<Option<Vec<u8>>>; 8]>
}

In the above example you can also see how arrays will be handled, the outside Option around the whole array will be some if atleast one of the items has changes or its attributes have changed, might need to add another function to the Networked trait to determine if a struct has changed

## Handling client events
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
