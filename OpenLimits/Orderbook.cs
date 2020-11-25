namespace OpenLimits {
    using System.Collections.Generic;
    public class Orderbook {
        private readonly Dictionary<double, double> _bids = new Dictionary<double, double>();
        private readonly Dictionary<double, double> _asks = new Dictionary<double, double>();
        
        public void Update(OrderbookResponse changes) {
            foreach(var ask in changes.asks) {
                if (ask.qty == 0){
                    if (_asks.ContainsKey(ask.price)) {
                        _asks.Remove(ask.price);
                    }
                } else {
                    _asks.Add(ask.price, ask.qty);
                }
            }

            foreach(var bid in changes.bids) {
                if (bid.qty == 0){
                    if (_bids.ContainsKey(bid.price)) {
                        _bids.Remove(bid.price);
                    }
                } else {
                    _bids.Add(bid.price, bid.qty);
                }
            }
        }
    }
}