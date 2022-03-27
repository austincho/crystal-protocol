import { LCDClient } from '@terra-money/terra.js'
import { contractAddress } from './address'

export const getOptionContract = async (wallet) => {
  const lcd = new LCDClient({
    URL: wallet.network.lcd,
    chainID: wallet.network.chainID,
  })
  return lcd.wasm.contractQuery(contractAddress(wallet), { get_option_contract: {} })
}
