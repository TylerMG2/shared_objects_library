/* 
 * The goal with this macro is to generate the necessary implementations and structs to allow for automatic networking of structs.
 * 
 * This needs a few things:
 * 
 * An option version of the struct where all its fields are Option<T> and non primitive types are replaced with Vec<u8> to be deserialized later
 * in their own implementation of Networked. This makes the most sense since when we deserialize incoming data its much easier to then pass each 
 * vec<u8> to the correct struct to be deserialized.
 * 
 * We also need a way of tracking which fields have been updated since last sync, this can be done a few ways:
 *  - We can define a generic struct that each field needs to be wrapped in that tracks if the field has been updated, this would be the most efficient but requires more boilerplate
 *  - We can have the macro generate a wrapper around the struct that has 2 versions of the struct, one called previous and one called current, when trying to send the struct
 *  we can compare the 2 and only send the fields that have changed, this is less efficient but requires less boilerplate and is easier to implement
 * 
 * 
 */