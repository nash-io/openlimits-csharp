using NUnit.Framework;
namespace OpenLimitsTest
{
    using OpenLimits;
    using System;
    public class ExchangeClientTest
    {
        [Test]
        public void TestBinance()
        {
            BinanceClientConfig config = BinanceClientConfig.Authenticated(
                Environment.GetEnvironmentVariable("BINANCE_API_KEY"),
                Environment.GetEnvironmentVariable("BINANCE_SECRET"),
                true
            );
            
            var client = new ExchangeClient(config);
            TestContext.Progress.WriteLine("Testing error handling");

            Assert.Throws<BinanceError>(
                delegate { 
                    client.GetHistoricRates(new GetHistoricRatesRequest("sadsdqwe", "OneHour"));
                }
            );

            TestContext.Progress.WriteLine("Orderbook " + client.Orderbook("BNBBTC"));
            TestContext.Progress.WriteLine("GetAccountBalances " + client.GetAccountBalances());
            TestContext.Progress.WriteLine("GetHistoricRates: " + client.GetHistoricRates(new GetHistoricRatesRequest("BNBBTC", "OneHour")));
            TestContext.Progress.WriteLine("GetHistoricRates: " + client.GetHistoricRates(new GetHistoricRatesRequest("BNBBTC", "OneHour")));
            TestContext.Progress.WriteLine("GetAllOpenOrders: " + client.GetAllOpenOrders());
            TestContext.Progress.WriteLine("GetOrderHistory BNBBTC: " + client.GetOrderHistory(new GetOrderHistoryRequest("BNBBTC")));
            TestContext.Progress.WriteLine("Limit buy: " + client.LimitBuy(LimitOrderRequest.goodTillCancelled("0.001", "1", "BNBBTC")));
            TestContext.Progress.WriteLine("Limit sell: " + client.LimitSell(LimitOrderRequest.goodTillCancelled("0.001", "1", "BNBBTC")));
            TestContext.Progress.WriteLine("Limit buy fok: " + client.LimitBuy(LimitOrderRequest.fillOrKill("0.001", "1", "BNBBTC")));
            TestContext.Progress.WriteLine("Limit buy ioc: " + client.LimitBuy(LimitOrderRequest.immediateOrCancel("0.001", "1", "BNBBTC")));
            TestContext.Progress.WriteLine("Market buy: " + client.MarketBuy(new MarketOrderRequest("1", "BNBBTC")));
            TestContext.Progress.WriteLine("Market sell: " + client.MarketSell(new MarketOrderRequest("1", "BNBBTC")));
        }
    }
}