import ps from 'process';

const ENV = ps.env;

export interface AppConfig {
  SOLANA_RPC: string;
  SOLANA_NETWORK: string;
}


async function loadConfig(): Promise<AppConfig> {

  const config:AppConfig = {
    SOLANA_RPC: ENV.SOLANA_RPC || 'abc',
    SOLANA_NETWORK: ENV.SOLANA_NETWORK || 'devnet',
  }

  console.log(config);
  return config;
}


export const CONFIG = loadConfig();
