namespace OpenLimits
{
    using System.Collections.Generic;

    public class OrderbookResponse
    {
        readonly public string market;
        readonly public IEnumerable<AskBid> asks;
        readonly public IEnumerable<AskBid> bids;

        public OrderbookResponse(string market, IEnumerable<AskBid> asks, IEnumerable<AskBid> bids)
        {
            this.market = market;
            this.asks = asks;
            this.bids = bids;
        }
    }
}