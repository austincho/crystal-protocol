# Crystal Protocol


<p align="center" >
<img src="./docs/logo.svg" alt="logo" width="200"/>
</p>

<p align="center" >
<b>Look into the crystal ball and hedge your bets with option contracts on the Terra Blockchain!</b>
</p>

---

##Introduction
Given that the Terra Ecosystem currently has many excellent DEFI Projects in development, I wanted to develop something that 
I have not seen in the ecosystem yet. We currently have Anchor that acts as a Bank facilitating borrowing/lending, we also 
have Mirror, that offers the ability to purchase/sell tokenized securities, but I have yet to see a protocol that provides #lunatics
with the opportunity to participate in option contracts in the Terra Ecosystem.


### So what is an Options Contract?
An options Contract is an agreement between two parties to facilitate a potential 
transaction involving an asset at a preset price prior and date. Options are generally used for hedging purposes but also can be used for
speculation as well. 


###Types of Options:

**Call Option Contract** 
- The buyer of a call option has the ability to purchase the underlying security from the other party at specified price by the expiration date.
- The seller of a call option provides the buyer with the opportunity to buy the underlying security at a specific price till the expiration date, in exchange they receive a premium.

You typically buy Call Options if you are bullish on an asset, as you believe that the price of the underlying security will appreciate more than the agreed upon specified price. As a result you can buy the
underlying security from the seller of the Call Option contract and then sell it on the market.

**Profit = Market Price - Specified Price**

**Put Option Contract** 
- The buyer of a put option has the ability to sell the underlying security to the other party at specified price by the expiration date.
- The seller of a put option provides the buyer with the opportunity to sell the underlying security at a specific price till the expiration date, in exchange they receive a premium.

You typically buy Put Options if you are bearish on an asset, as you believe that the underlying security will depreciate more than the agreed upon specified price. As a result you can buy the underlying
security on the market and then sell the security to the seller of the Put Option contract.

**Profit = Specified Price - Market Price**

## Implementation

**NOTE: I may refer to the buyer of the contract as the "holder" and seller of the contract as the "underwriter."**

### State
The Option Smart Contract stores this information
```
pub enum OptionStatus {
    CREATED,       
    FUNDED,         
    LOCKED,
    EXECUTED,
}

pub struct State {
    pub option_status: OptionStatus,    // The stage of the Option Contract
    pub creator: Addr,              
    pub holder: Addr,                   // The buyer of a Call/Put Option
    pub underwriter: Option<Addr>,      // The seller of a Call/Put Option
    pub asset: Vec<Coin>,               // The asset the holder has the option to buy
    pub collateral: Vec<Coin>,          // The price the buyer will pay for the option
    pub premium: Vec<Coin>,             // The premium that will be paid to the seller
    pub expires: u64,                   // The block height the option is valid till
}
```

### InstantiateMsg

In order to first create an Options Contract, the buyer(aka the holder) will send an InstantiateMsg...
```
\\ Example of Call Contract
let msg = InstantiateMsg {
            asset: coins(100000, "uluna"),       //1000 Luna
            collateral: coins(100000, "uusd"),   //1000 USD
            premium: coins(10000, "uusd"),       //100  USD
            expires: 10000                       //The block height the option is valid till
        };
        
\\ Example of Put Contract
let msg = InstantiateMsg {
            asset: coins(100000, "uusd"),       //1000 USD
            collateral: coins(100000, "uluna"),   //1000 LUNA
            premium: coins(10000, "uusd"),       //100  USD
            expires: 10000                       //The block height the option is valid till
        };
```
As you can see...
* The collateral field as what the buyer(aka the "holder") is putting into the contract and if the
contract is executed the collateral is transferred to the seller(aka the "underwriter").

* The asset field as what the 
seller(aka the "underwriter") of the option is putting into the contract and if the contract is executed the asset will be 
transferred to the buyer(aka the "holder").

### ExecuteMsg

```
pub enum ExecuteMsg {
    FundOption {},
    UnderwriteOption { underwrite_option_req: UnderwriteOptionRequest },
    ExecuteOption {},
    TransferOption {recipient : Addr},
    WithdrawExpiredOption {},
    WithdrawUnlockedOption {},
}
```
There are four stages to the Options Contract...

```
pub enum OptionStatus {
    CREATED,    - Terms of the contract has been specified but not funded yet.
    FUNDED,     - The holder has funded the specified premium and collateral.
    LOCKED,     - The underwriter has funded the specified asset. Premium has been transferred to underwriter.
    EXECUTED,   - The holder has executed the option. collateral transferred to underwriter and asset transferred to holder.
}
```

#### Fund Option Contract - as holder
* After instantiating the smart contract the `holder` of the contract will have to fund the option by sending an `ExecuteMsg::FundOption` msg
and send the specified `premium` and `collateral`. If funded with the correct amount the contract will move to `OptionStatus::FUNDED` stage.

#### Fund Option Contract - as underwriter
* Once the contract has moved to `OptionStatus::FUNDED` stage the `underwriter` will be able to interact with the smart contract by
sending an `ExecuteMsg::UnderwriteOption` msg. They will send along `UnderwriteOptionRequest` object that sets out the terms of the options agreement and the asset
required for the option contract.
* If the data in `UnderwriteOptionRequest` agrees with the InstantiateMsg sent from the 'holder' then the `underwriter` will have the `premium` transferred
to their wallet. The stage of the contract will move to `OptionStatus::LOCKED`.

#### Execute Option Contract - as holder
* At any time before the specified block height the `holder` can execute the option. When executed the `collateral` will be sent to 
the `underwriter` and the `asset` will be transferred to the 'holder'. The stage of the contract will move to `OptionStatus::EXECUTED`.


You also have the ability to transfer the option contract with `ExecuteMsg::TransferOption` msg and withdraw the `asset` and `collateral`
back to their rightful owners if the option was not executed and has expired.
