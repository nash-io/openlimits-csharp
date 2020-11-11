using System;

namespace OpenLimitsTestApp
{
    using OpenLimits;
    using System.Threading;
    class Program
    {
        static public void PrintBook(OrderbookResponse orderbook) {
            Console.WriteLine("New orderbook orders in " + orderbook.market);
            Console.WriteLine("asks");
            foreach(var ask in orderbook.asks) {
                Console.WriteLine(ask);
            }

            Console.WriteLine("bids");
            foreach(var bid in orderbook.bids) {
                Console.WriteLine(bid);
            }
        }

        static public void Main(string[] args)
        {
            var secret = "";
            var apikey = "";
            
            NashClientConfig config = NashClientConfig.Authenticated(apikey, secret, 0, NashEnvironment.Sandbox, 1000);
            var client = new ExchangeClient(config);

            Console.WriteLine("Available markets");
            foreach(var market in client.ReceivePairs()) {
                Console.WriteLine("Market: " + market.quote);
            }
            
            Console.WriteLine("Listening to the btc_usdc market");
            client.SubscribeToOrderbook("btc_usdc", PrintBook);
            
            // Noia markets only available in NashEnvironment.Production
            // Console.WriteLine("Listening to the noia markets");
            // client.SubscribeToOrderbook("noia_usdc", PrintBook);
            // client.SubscribeToOrderbook("noia_btc", PrintBook);
        }
    }
}