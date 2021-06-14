# VNRS (The Vanity Name Registration System)

## Task
The purpose of the name is outside the scope of the assignment and you can make reasonable assumptions on the size, encoding, etc of the name to complete in time. An unregistered name can be registered for a certain amount of time by locking a certain balance of an account. After the registration expires, the account loses ownership of the name and his balance is unlocked. The registration can be renewed by making an on-chain call to keep the name registered and balance locked. You can assume reasonable defaults for the locking amount and period. The fee to register the name depends directly on the size of the name. Also, a malicious node/validator should not be able to front-run the process by censoring transactions of an honest user and registering its name in its own account.

## Note
Since the validator has complete information about the actions performed in the transaction, we need an irreversible transformation - a hash. However, it is inconvenient when the information about the name is not in the clear text on the storage. Therefore, we introduce two actions:
 - Create a reservation for a name by the hash 
 - Register a name based on reservation

In general, the task is cool, I liked solving it.
I haven't worked on Substrate for about a year, the ecosystem has gotten better, but there are still a lot of nuances that need to be worked on (as example the template was not complied due to minor errors in dependencies).

I didn't have time to polish many things, but I think the code conveyed the main essence. ✌️

## TODO
Didn't manage to do it in the allotted time:
 - Normal lock management (now using mock for getting lock id)
 - Tests for check token locks
 - Refactoring
     - Not using tuple inside storage, for more clean usage
     - Add some rustdoc for API and internal functions & types
     - Rename `reservation` to `booking` or something easier to understand
 - Clean up substrate template
 - High level description about pallet logic and API

