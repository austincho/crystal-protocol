import './App.css'

import { useEffect, useState } from 'react'
import {
  useWallet,
  useConnectedWallet,
  WalletStatus,
} from '@terra-money/wallet-provider'

import * as execute from './contract/execute'
import * as query from './contract/query'
import { ConnectWallet } from './components/ConnectWallet'
import {Coin, Coins, Fee} from "@terra-money/terra.js";

function App() {
  const [option, setOption] = useState(null);
  const [isHolder, setHolder] = useState(false);
  const [isUnderwriter, setUnderwriter] = useState(false);
  const [updating, setUpdating] = useState(true);

  let optionStatus = "L";

  const { status } = useWallet()

  const connectedWallet = useConnectedWallet()

  useEffect(() => {
    const prefetch = async () => {
      if (connectedWallet) {
        const result = await query.getOptionContract(connectedWallet);
        setOption(result.state)
        console.log(result.state)
      }
      setUpdating(false)
    }
    prefetch()
  }, [])


  const onClickFundCollateral = async() => {
    setUpdating(true);
    const coin = new Coin('uusd', 10);
    let result = await execute.fund(connectedWallet, new Fee(200000, { uluna: 10000 }),{fund_collateral: {}}, new Coins([coin]) );
    debugger;
  }

  return (
    <div className="App">
      <header className="App-header">
        <div style={{ display: 'inline' }}>
          {optionStatus}
        </div>
        {status === WalletStatus.WALLET_CONNECTED && (
          <div style={{ display: 'inline' }}>
            <button onClick={onClickFundCollateral} type="button">
              Fund Collateral
            </button>
            <button onClick={onClickFundCollateral} type="button">
              Fund Premium
            </button>
            <button onClick={onClickFundCollateral} type="button">
              Execute
            </button>
            <button onClick={onClickFundCollateral} type="button">
              Withdraw
            </button>
          </div>
        )}
        <ConnectWallet />
      </header>
    </div>
  )
}

export default App
