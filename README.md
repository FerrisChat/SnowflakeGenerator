This library generates snowflakes based off Twitter's snowflake format with some modifications.

https://github.com/twitter-archive/snowflake/tree/snowflake-2010

# Changes
* unsigned 128 bit integers are used
* atomic 16 bit counters are used to allow up to 65,536 IDs to be generated every millisecond
* response time must be less than 5 microseconds

# Format
* Bits 0 to 63: milliseconds since the Ferris Epoch (01/01/2020 00:00:00.0000+00:00).
Range of around 600,000,000 years.
* Bits 64 to 71: the type of model (i.e. user, channel, guild)
* Bits 73 to 85: internal 16-bit atomic counter
* Bits 86 to 93: the API version this ID was generated with
* Bits 94 to 109: the node this ID was generated on
* Bits 110 to 127: unused

# Crate Features
* `time-safety-checks`: checks that the system clock has not rolled back since the last
snowflake generated and if it has, blocks until the time is after the time of the last snowflake.
Adds a slight performance penalty but isn't that noticeable. Enabled by default.