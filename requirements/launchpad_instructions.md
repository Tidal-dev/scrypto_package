#Project overview
Use this guide to build a token launchpad smart contract.

#Feature requirements
-We will use the scrypto lenguage to build on the radix blockchain
-the smart contract will be a blueprint that intantiates launchpads by the owner of the contract
-the launchpad is a simple fixed price ICO style
-each launchpad is instantiated with the parameters: start_time, end_time, sold_token, buy_token, price
-when users buy during the sale time they get registered but don't get the sold_token yet
-after the end of the sale time users can claim their tokens 