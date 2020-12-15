using NUnit.Framework;
namespace OpenLimitsTest
{
    using OpenLimits;
    using System;
    using System.Diagnostics;
    public class ExchangeClientTest
    {
        [Test]
        public void TestBinance()
        {   
            
            BinanceClientConfig config = BinanceClientConfig.Authenticated(
                Environment.GetEnvironmentVariable("BINANCE_API_KEY"),
                Environment.GetEnvironmentVariable("BINANCE_API_SECRET"),
                true
            );
            
            var client = new ExchangeClient(config);

            TestContext.Progress.WriteLine("Testing error handling");
            Assert.Throws<BinanceError>(
                delegate { 
                    client.GetHistoricRates(new GetHistoricRatesRequest("sadsdqwe", Interval.OneHour));
                }
            );

            TestContext.Progress.WriteLine("Orderbook " + client.Orderbook("BNBBTC"));
            TestContext.Progress.WriteLine("GetAccountBalances " + client.GetAccountBalances());
            TestContext.Progress.WriteLine("GetHistoricRates: " + client.GetHistoricRates(new GetHistoricRatesRequest("BNBBTC", Interval.OneHour)));
            TestContext.Progress.WriteLine("GetHistoricRates: " + client.GetHistoricRates(new GetHistoricRatesRequest("BNBBTC", Interval.OneHour)));
            TestContext.Progress.WriteLine("GetAllOpenOrders: " + client.GetAllOpenOrders());
            TestContext.Progress.WriteLine("GetOrderHistory BNBBTC: " + client.GetOrderHistory(new GetOrderHistoryRequest("BNBBTC")));
            TestContext.Progress.WriteLine("Limit buy: " + client.LimitBuy(LimitOrderRequest.goodTillCancelled("0.001", "1", "BNBBTC")));
            TestContext.Progress.WriteLine("Limit sell: " + client.LimitSell(LimitOrderRequest.goodTillCancelled("0.001", "1", "BNBBTC")));
            TestContext.Progress.WriteLine("Limit buy fok: " + client.LimitBuy(LimitOrderRequest.fillOrKill("0.001", "1", "BNBBTC")));
            TestContext.Progress.WriteLine("Limit buy ioc: " + client.LimitBuy(LimitOrderRequest.immediateOrCancel("0.001", "1", "BNBBTC")));
            TestContext.Progress.WriteLine("Market buy: " + client.MarketBuy(new MarketOrderRequest("1", "BNBBTC")));
            TestContext.Progress.WriteLine("Market sell: " + client.MarketSell(new MarketOrderRequest("1", "BNBBTC")));
        }

        [Test]
        public void TestNash()
        {
            NashClientConfig config = NashClientConfig.Authenticated(
                Environment.GetEnvironmentVariable("NASH_API_KEY"),
                Environment.GetEnvironmentVariable("NASH_API_SECRET"),
                0,
                NashEnvironment.Sandbox,
                1000
            );
            
            var client = new ExchangeClient(config);

            TestContext.Progress.WriteLine("Orderbook " + client.Orderbook("btc_usdc"));
            TestContext.Progress.WriteLine("ReceivePairs " + client.ReceivePairs());
            TestContext.Progress.WriteLine("GetAccountBalances " + client.GetAccountBalances());
            TestContext.Progress.WriteLine("GetHistoricRates: " + client.GetHistoricTrades(new GetHistoricTradesRequest("btc_usdc")));
            TestContext.Progress.WriteLine("GetOrderHistory btc_usdc: " + client.GetOrderHistory(new GetOrderHistoryRequest("btc_usdc")));
            client.CancelAllOrders("btc_usdc");
            var order = client.LimitSell(LimitOrderRequest.goodTillCancelled("6500.0", "0.01000", "btc_usdc"));
            
            TestContext.Progress.WriteLine("Limit sell: " + order);
            TestContext.Progress.WriteLine("Get order", client.GetOrder(order.id, order.marketPair));
            TestContext.Progress.WriteLine("Limit buy: " + client.LimitBuy(LimitOrderRequest.goodTillCancelled("6500.0", "0.01000", "btc_usdc")));
            TestContext.Progress.WriteLine("Limit buy fok: " + client.LimitSell(LimitOrderRequest.fillOrKill("6500.0", "0.01000", "btc_usdc")));
            TestContext.Progress.WriteLine("Limit buy ioc: " + client.LimitSell(LimitOrderRequest.immediateOrCancel("6500.0", "0.01000", "btc_usdc")));
            TestContext.Progress.WriteLine("Market sell: " + client.MarketSell(new MarketOrderRequest("0.01000", "btc_usdc")));
            TestContext.Progress.WriteLine("Market sell inverse: " + client.MarketSell(new MarketOrderRequest("20", "usdc_btc")));
            client.CancelAllOrders("btc_usdc");
        }

        [Test]
        public void TestCoinbase()
        {
            CoinbaseClientConfig config = CoinbaseClientConfig.Authenticated(
                Environment.GetEnvironmentVariable("COINBASE_API_KEY"),
                Environment.GetEnvironmentVariable("COINBASE_API_SECRET"),
                Environment.GetEnvironmentVariable("COINBASE_PASSPHRASE"),
                true
            );
            
            var client = new ExchangeClient(config);

            TestContext.Progress.WriteLine("Orderbook " + client.Orderbook("ETH-BTC"));
            TestContext.Progress.WriteLine("ReceivePairs " + client.ReceivePairs());
            TestContext.Progress.WriteLine("GetAccountBalances " + client.GetAccountBalances());
            TestContext.Progress.WriteLine("GetOrderHistory ETH-BTC: " + client.GetOrderHistory(new GetOrderHistoryRequest("ETH-BTC")));
            client.CancelAllOrders("ETH-BTC");
            TestContext.Progress.WriteLine("Limit sell: " + client.LimitSell(LimitOrderRequest.goodTillCancelled("0.001", "1", "ETH-BTC")));
            TestContext.Progress.WriteLine("Limit buy: " + client.LimitBuy(LimitOrderRequest.goodTillCancelled("0.001", "1", "ETH-BTC")));
            // TestContext.Progress.WriteLine("Limit buy fok: " + client.LimitSell(LimitOrderRequest.fillOrKill("0.001", "1", "ETH-BTC")));
            TestContext.Progress.WriteLine("Limit buy ioc: " + client.LimitSell(LimitOrderRequest.immediateOrCancel("0.001", "1", "ETH-BTC")));
            client.CancelAllOrders("ETH-BTC");
        }
    }
}