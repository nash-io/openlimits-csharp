
namespace OpenLimits
{
    
    using System;
    using System.Threading;
    using System.Collections.Generic;
    using System.Runtime.InteropServices;
    public class ExchangeClient
    {
        private void handleResult(FFIResult result) {
            string message = "Unknown error";
            if (result.message.ToInt64() != 0) {
                message = CString.ToString(result.message);
                FreeString(result.message);
            }
            switch(result.tag) {
                case ResultTag.Ok: return;
                case ResultTag.InvalidArgument:
                    throw new ArgumentException(message);
                case ResultTag.BinanceError:
                    throw new BinanceError(message);
                case ResultTag.CoinbaseError:
                    throw new CoinbaseError(message);
                case ResultTag.NashProtocolError:
                    throw new NashProtocolError(message);
                case ResultTag.MissingImplementation:
                    throw new MissingImplementation(message);
                case ResultTag.AssetNotFound:
                    throw new AssetNotFound(message);
                case ResultTag.NoApiKeySet:
                    throw new NoApiKeySet(message);
                case ResultTag.InternalServerError:
                    throw new InternalServerError(message);
                case ResultTag.ServiceUnavailable:
                    throw new ServiceUnavailable(message);
                case ResultTag.Unauthorized:
                    throw new Unauthorized(message);
                case ResultTag.SymbolNotFound:
                    throw new SymbolNotFound(message);
                case ResultTag.SocketError:
                    throw new SocketError(message);
                case ResultTag.GetTimestampFailed:
                    throw new GetTimestampFailed(message);
                case ResultTag.ReqError:
                    throw new ReqError(message);
                case ResultTag.InvalidHeaderError:
                    throw new InvalidHeaderError(message);
                case ResultTag.InvalidPayloadSignature:
                    throw new InvalidPayloadSignature(message);
                case ResultTag.IoError:
                    throw new IoError(message);
                case ResultTag.PoisonError:
                    throw new PoisonError(message);
                case ResultTag.JsonError:
                    throw new JsonError(message);
                case ResultTag.ParseFloatError:
                    throw new ParseFloatError(message);
                case ResultTag.UrlParserError:
                    throw new UrlParserError(message);
                case ResultTag.Tungstenite:
                    throw new Tungstenite(message);
                case ResultTag.TimestampError:
                    throw new TimestampError(message);
                case ResultTag.UnkownResponse:
                    throw new UnkownResponse(message);
                case ResultTag.NotParsableResponse:
                    throw new NotParsableResponse(message);
                case ResultTag.MissingParameter:
                    throw new MissingParameter(message);     
                case ResultTag.WebSocketMessageNotSupported:
                    throw new WebSocketMessageNotSupported(message);            }
        }
        /// Used by rust to write data directly to C# thus avoiding changing ownership
        private FFITrade[] subTradesBuff = new FFITrade[1024];
        private AskBid[] subAsksBuff = new AskBid[1024];
        private AskBid[] subBidsBuff = new AskBid[1024];

        // Callbacks from rust into C#. Some callbacks come in a "private" and public version.
        // Some objects, especially those containing strings or array of objects will be serialized into a
        // C# version after arriving. Strings exchanged from rust to C# must be freed manually. So it is important not to expose
        // The internals
        public delegate void OnError();
        public delegate void OnPing();
        private delegate void OnDisconnect();
        public delegate void OnOrderbook(OrderbookResponse orderbook);
        unsafe private delegate void OnOrderbookFFI(ulong bidActualValueLen, ulong askActualValueLen, IntPtr market);
        public delegate void OnTrades(TradesResponse trades);
        private delegate void OnTradesFFI(ulong bidActualValueLen, IntPtr market);
        private OnError onErrorCB;

        private OnPing onPingCB;

        private Dictionary<string, List<OnOrderbook>> onOrderbookCbs = new Dictionary<string, List<OnOrderbook>>();
        private Dictionary<string, List<OnTrades>> onTradesCbs = new Dictionary<string, List<OnTrades>>();

       
        const string NativeLib = "libopenlimits_sharp";

        unsafe private void* _client_handle;
        unsafe private IntPtr _sub_handle;

        [DllImport(NativeLib, EntryPoint = "free_string", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe internal static extern void FreeString(IntPtr handle);

        
        [DllImport(NativeLib, EntryPoint = "disconnect", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe internal static extern void Disconnect(IntPtr subhandle);

        [DllImport(NativeLib, EntryPoint = "init_binance", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern void* InitBinance(BinanceClientConfig config);

        [DllImport(NativeLib, EntryPoint = "init_nash", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern void* InitNash(string apikey, string secret, ulong clientid, NashEnvironment environment, ulong timeout);
        
        
        [DllImport(NativeLib, EntryPoint = "init_subscriptions", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern IntPtr InitCbs(void* client,
            OnError onError, OnPing onPing, OnOrderbookFFI onOrderbook, OnTradesFFI onTrades, OnDisconnect onDisconnect,
            IntPtr bidBuffPtr, UIntPtr bidBufLen,
            IntPtr askBuffPtr, UIntPtr askBufLen,
            IntPtr taskBuffPtr, UIntPtr tradeBufLen
        );


        [DllImport(NativeLib, EntryPoint = "order_book", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult Orderbook(void* client, string market,
            IntPtr bidBuffPtr, UIntPtr bidBufLen, out UIntPtr bidActualValueLen,
            IntPtr askBuffPtr, UIntPtr AskBufLen, out UIntPtr askActualValueLen
        );

        [DllImport(NativeLib, EntryPoint = "get_price_ticker", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult GetPriceTicker(void* client, string market, out double price);

        [DllImport(NativeLib, EntryPoint = "get_historic_rates", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult GetHistoricRates(void* client, string market, string interval, Paginator paginator,
            IntPtr buffPtr, UIntPtr valueBufLen, out UIntPtr actualValueLen
        );

        [DllImport(NativeLib, EntryPoint = "get_historic_trades", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult GetHistoricTrades(void* client, string market, Paginator paginator,
            IntPtr buffPtr, UIntPtr valueBufLen, out UIntPtr actualValueLen
        );

        [DllImport(NativeLib, EntryPoint = "place_order", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult PlaceOrder(void* client, string market,
            string qty,
            bool limit,
            string price,
            Side side,
            TimeInForce tif,
            ulong tifDuration,
            bool postOnly,
            out FFIOrder order
        );
        
        [DllImport(NativeLib, EntryPoint = "get_all_open_orders", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult GetAllOpenOrders(void* client,
            IntPtr buffPtr, UIntPtr valueBufLen, out UIntPtr actualValueLen
        );

        [DllImport(NativeLib, EntryPoint = "subscribe_orderbook", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult SubscribeToOrderbook(IntPtr subhandle, string market);

        [DllImport(NativeLib, EntryPoint = "subscribe_trades", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult SubscribeToTrades(IntPtr subhandle, string market);

        [DllImport(NativeLib, EntryPoint = "get_order_history", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult GetOrderHistory(void* client,
            string market, Paginator paginator,
            IntPtr buffPtr, UIntPtr valueBufLen, out UIntPtr actualValueLen
        );

        [DllImport(NativeLib, EntryPoint = "get_trade_history", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult GetTradeHistory(void* client,
            string market, string orderId, Paginator paginator,
            IntPtr buffPtr, UIntPtr valueBufLen, out UIntPtr actualValueLen
        );

        [DllImport(NativeLib, EntryPoint = "get_account_balances", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult GetAccountBalances(void* client,
            Paginator paginator,
            IntPtr buffPtr, UIntPtr valueBufLen, out UIntPtr actualValueLen
        );


        [DllImport(NativeLib, EntryPoint = "cancel_all_orders", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult CancelAllOrders(void* client, string market, IntPtr buffPtr, UIntPtr valueBufLen, out UIntPtr actualValueLen);

        [DllImport(NativeLib, EntryPoint = "receive_pairs", ExactSpelling = true, CallingConvention = CallingConvention.Cdecl)]
        unsafe private static extern FFIResult ReceivePairs(void* client, IntPtr buffPtr, UIntPtr valueBufLen, out UIntPtr actualValueLen);

        private void handleFFIResult(FFIResult result) {
        }
        private void onPingHandler() {
            if (this.onPingCB == null){
                return;
            }

            onPingCB();
        }
        private void onErrorHandler() {
            if (this.onErrorCB == null){
                return;
            }

            onErrorCB();
        }
        unsafe private void onTradesHandler(ulong tradeBuffLen, IntPtr marketStr) {
            var market = CString.ToString(marketStr);
            FreeString(marketStr);
            var tradesList = new List<Trade>();
            
            for (int i = 0 ; i < (int)tradeBuffLen ; i ++) {
                tradesList.Add(subTradesBuff[i].ToTrade());
                subTradesBuff[i].Dispose();
            }

            if (!onTradesCbs.ContainsKey(market)) {
                return;
            }
            var trades = new TradesResponse(market, tradesList);
            this.onTradesCbs.TryGetValue(market, out var callbacks);
            foreach(var callback in callbacks) {
                callback(trades);
            }
        }
        unsafe private void onOrderbookHandler(ulong bidActualValueLen, ulong askActualValueLen, IntPtr marketStr) {
            var market = CString.ToString(marketStr);
            FreeString(marketStr);
           
            var bidsList = new List<AskBid>();
            var asksList = new List<AskBid>();

            
            for (int i = 0 ; i < (int)bidActualValueLen ; i ++) {
                bidsList.Add(subBidsBuff[i]);
            }
            
            for (int i = 0 ; i < (int)askActualValueLen ; i ++) {
                asksList.Add(subAsksBuff[i]);
            }

            if (!onOrderbookCbs.ContainsKey(market)) {
                return;
            }
            var latestOrderbook = new OrderbookResponse(
                market,
                asksList,
                bidsList
            );

            this.onOrderbookCbs.TryGetValue(market, out var callbacks);
            foreach(var callback in callbacks) {
                callback(latestOrderbook);
            }
        }
        EventWaitHandle ewh = new EventWaitHandle(false, EventResetMode.ManualReset);
        Thread ewhThreadHandle = null;
        private void onDisconnect() {
            ewh.Set();
        }

        unsafe private IntPtr InitCbs() {
            fixed (AskBid* bidBuff = subBidsBuff.AsSpan()) {
                fixed (AskBid* askBuff = subAsksBuff.AsSpan()) {
                    fixed (FFITrade* tradeBuff = subTradesBuff.AsSpan()) {
                        return InitCbs(
                            _client_handle,
                            this.onPingHandler,
                            this.onErrorHandler,
                            this.onOrderbookHandler,
                            this.onTradesHandler,
                            this.onDisconnect,

                            (IntPtr)bidBuff, (UIntPtr)subBidsBuff.Length,
                            (IntPtr)askBuff, (UIntPtr)subAsksBuff.Length,
                            (IntPtr)tradeBuff, (UIntPtr)subTradesBuff.Length
                        );
                    }
                }
            }
        }

        unsafe public ExchangeClient(BinanceClientConfig config) {
            _client_handle = ExchangeClient.InitBinance(config);
            _sub_handle = InitCbs();
        }

        unsafe public ExchangeClient(NashClientConfig config) {
            _client_handle = ExchangeClient.InitNash(config.apikey, config.secret, config.clientId, config.environment, config.timeout);
            _sub_handle = InitCbs();
        }

        unsafe public double GetPriceTicker(string market) {
            var result = ExchangeClient.GetPriceTicker(_client_handle, market, out double price);
            return price;
        }
        unsafe public OrderbookResponse Orderbook(string market) {
            var bids = new AskBid[512];
            var asks = new AskBid[512];
            var bidsLen = bids.Length;
            var asksLen = asks.Length;
            var bidsList = new List<AskBid>();
            var asksList = new List<AskBid>();
            

            fixed (AskBid* bidBuff = bids.AsSpan()) {
                fixed (AskBid* askBuff = asks.AsSpan()) {
                    handleResult(ExchangeClient.Orderbook(
                        _client_handle,
                        market,
                        (IntPtr)bidBuff, (UIntPtr)bidsLen, out var actualBidsLen,
                        (IntPtr)askBuff, (UIntPtr)asksLen, out var actualAsksLen
                    ));
                    for (int i = 0 ; i < Math.Min(bidsLen, (int)actualBidsLen) ; i ++) {
                        bidsList.Add(bids[i]);
                    }
                    for (int i = 0 ; i < Math.Min(asksLen, (int)actualAsksLen) ; i ++) {
                        asksList.Add(asks[i]);
                    }
                }
            }

            return new OrderbookResponse(
                market,
                asksList,
                bidsList
            );
        }

         unsafe public IEnumerable<Candle> GetHistoricRates(GetHistoricRatesRequest req) {
            var limit = req.paginator == null ? 0 : req.paginator.limit;
            var candles = new Candle[Math.Max(limit, 256)];
            var candlesLen = candles.Length;
            var candlesList = new List<Candle>();
            

            fixed (Candle* candleBuff = candles.AsSpan()) {
                handleResult(ExchangeClient.GetHistoricRates(
                    _client_handle,
                    req.market, req.interval, req.paginator,
                    (IntPtr)candleBuff, (UIntPtr)candlesLen, out var actualCandleLen
                ));
                for (int i = 0 ; i < (int)actualCandleLen ; i ++) {
                    candlesList.Add(candles[i]);
                }
            }

            return candlesList;
        }
        unsafe public IEnumerable<Trade> GetHistoricTrades(GetHistoricTradesRequest req) {
            var limit = req.paginator == null ? 0 : req.paginator.limit;
            var trades = new FFITrade[Math.Max(limit, 256)];
            var tradesLen = trades.Length;
            var tradesList = new List<Trade>();
            

            fixed (FFITrade* tradeBuff = trades.AsSpan()) {
                handleResult(ExchangeClient.GetHistoricTrades(
                    _client_handle,
                    req.market,
                    req.paginator,
                    (IntPtr)tradeBuff, (UIntPtr)tradesLen, out var actualTradeLen
                ));
                for (int i = 0 ; i < (int)actualTradeLen ; i ++) {
                    tradesList.Add(trades[i].ToTrade());
                    trades[i].Dispose();
                }
            }

            return tradesList;
        }


        unsafe public Order LimitBuy(LimitOrderRequest request) {
            handleResult(ExchangeClient.PlaceOrder(
                _client_handle,
                request.market,
                request.size,
                true,
                request.price,
                Side.Buy,
                request.timeInForce,
                request.timeInForceDurationMs,
                request.postOnly,
                out FFIOrder ffiOrder
            ));
            var order = ffiOrder.ToOrder();
            ffiOrder.Dispose();
            return order;
        }
        unsafe public Order LimitSell(LimitOrderRequest request) {
            handleResult(ExchangeClient.PlaceOrder(
                _client_handle,
                request.market,
                request.size,
                true,
                request.price,
                Side.Sell,
                request.timeInForce,
                request.timeInForceDurationMs,
                request.postOnly,
                out FFIOrder ffiOrder
            ));
            var order = ffiOrder.ToOrder();
            ffiOrder.Dispose();
            return order;
        }

        unsafe public Order MarketBuy(MarketOrderRequest request) {
            handleResult(ExchangeClient.PlaceOrder(
                _client_handle,
                request.market,
                request.size,
                false,
                null,
                Side.Buy,
                TimeInForce.GTC,
                0,
                false,
                out FFIOrder ffiOrder
            ));
            var order = ffiOrder.ToOrder();
            ffiOrder.Dispose();
            return order;
        }

        unsafe public Order MarketSell(MarketOrderRequest request) {
            handleResult(ExchangeClient.PlaceOrder(
                _client_handle,
                request.market,
                request.size,
                false,
                null,
                Side.Sell,
                TimeInForce.GTC,
                0,
                false,
                out FFIOrder ffiOrder
            ));
            var order = ffiOrder.ToOrder();
            ffiOrder.Dispose();
            return order;
        }
        unsafe public IEnumerable<Order> GetAllOpenOrders() {
            var orders = new FFIOrder[256];
            var ordersLen = orders.Length;
            var ordersList = new List<Order>();
            

            fixed (FFIOrder* orderBuff = orders.AsSpan()) {
                handleResult(ExchangeClient.GetAllOpenOrders(
                    _client_handle,
                    (IntPtr)orderBuff, (UIntPtr)ordersLen, out var actualCandleLen
                ));
                for (int i = 0 ; i < (int)actualCandleLen ; i ++) {
                    ordersList.Add(orderBuff[i].ToOrder());
                    orderBuff[i].Dispose();
                }
            }

            return ordersList;
        }

        unsafe public IEnumerable<Order> GetOrderHistory(GetOrderHistoryRequest req) {
            var limit = req.paginator == null ? 0 : req.paginator.limit;
            var orders = new FFIOrder[Math.Max(limit, 256)];
            var ordersLen = orders.Length;
            var ordersList = new List<Order>();
            

            fixed (FFIOrder* orderBuff = orders.AsSpan()) {
                handleResult(ExchangeClient.GetOrderHistory(
                    _client_handle,
                    req.market, req.paginator,
                    (IntPtr)orderBuff, (UIntPtr)ordersLen, out var actualCandleLen
                ));
                for (int i = 0 ; i < (int)actualCandleLen ; i ++) {
                    ordersList.Add(orderBuff[i].ToOrder());
                    orderBuff[i].Dispose();
                }
            }

            return ordersList;
        }

        unsafe public IEnumerable<Trade> GetTradeHistory(GetTradeHistoryRequest req) {
            var limit = req.paginator == null ? 0 : req.paginator.limit;
            var trades = new FFITrade[Math.Max(limit, 256)];
            var tradesLen = trades.Length;
            var tradesList = new List<Trade>();
            

            fixed (FFITrade* tradeBuff = trades.AsSpan()) {
                handleResult(ExchangeClient.GetTradeHistory(
                    _client_handle,
                    req.market, req.orderId, req.paginator,
                    (IntPtr)tradeBuff, (UIntPtr)tradesLen, out var actualCandleLen
                ));
                for (int i = 0 ; i < (int)actualCandleLen ; i ++) {
                    tradesList.Add(tradeBuff[i].ToTrade());
                    tradeBuff[i].Dispose();
                }
            }

            return tradesList;
        }
    
        unsafe public IEnumerable<Balance> GetAccountBalances(Paginator paginator) {
            var limit = paginator == null ? 0 : paginator.limit;
            var balances = new FFIBalance[Math.Max(limit, 256)];
            var balancesLen = balances.Length;
            var balancesList = new List<Balance>();
            

            fixed (FFIBalance* balanceBuff = balances.AsSpan()) {
                handleResult(ExchangeClient.GetAccountBalances(
                    _client_handle,
                    paginator,
                    (IntPtr)balanceBuff, (UIntPtr)balancesLen, out var actualCandleLen
                ));
                for (int i = 0 ; i < (int)actualCandleLen ; i ++) {
                    balancesList.Add(balanceBuff[i].ToBalance());
                    balanceBuff[i].Dispose();
                }
            }

            return balancesList;
        }
        public IEnumerable<Balance> GetAccountBalances() {
            return this.GetAccountBalances(null);
        }

        unsafe public IEnumerable<string> CancelAllOrders(string market) {
            var orders = new IntPtr[1024];
            var ordersLen = orders.Length;
            var cancelledOrdersList = new List<String>();
            fixed (IntPtr* orderBuff = orders.AsSpan()) {
                 handleResult(ExchangeClient.CancelAllOrders(
                    _client_handle,
                    market,
                    (IntPtr)orderBuff, (UIntPtr)ordersLen, out var actualLen
                ));
                for (int i = 0 ; i < (int)actualLen ; i ++) {
                    cancelledOrdersList.Add(CString.ToString(orders[i]));
                    ExchangeClient.FreeString(orders[i]);
                }
            }
            return cancelledOrdersList;
        }

        unsafe public IEnumerable<MarketPair> ReceivePairs() {
            var marketPairs = new FFIMarketPair[1024];
            var marketPairsLen = marketPairs.Length;
            var pairs = new List<MarketPair>();
            fixed (FFIMarketPair* buff = marketPairs.AsSpan()) {
                 handleResult(ExchangeClient.ReceivePairs(
                    _client_handle,
                    (IntPtr)buff, (UIntPtr)marketPairsLen, out var actualLen
                ));
                for (int i = 0 ; i < (int)actualLen ; i ++) {
                    pairs.Add(marketPairs[i].ToMarketPair());
                    marketPairs[i].Dispose();
                }
            }
            return pairs;
        }

        private void WaitForEwh() {
            ewh.WaitOne();
        }

        private void SetupEWH() {
            if (ewhThreadHandle != null) {
                return;
            }

            ewhThreadHandle = new Thread(this.WaitForEwh);
            ewhThreadHandle.Start();
        }

        public void Listen(
            OnError onError,
            OnPing onPing
        ) {
            this.onErrorCB = onError;
            this.onPingCB = onPing;

            this.SetupEWH();

        }

        unsafe public void SubscribeToOrderbook(string market, OnOrderbook onOrderbook) {
            if (!this.onOrderbookCbs.ContainsKey(market)) {
                this.onOrderbookCbs.Add(market, new List<OnOrderbook>());
            }
            this.onOrderbookCbs.TryGetValue(market, out var callbacks);
            callbacks.Add(onOrderbook);
            handleFFIResult(SubscribeToOrderbook(this._sub_handle, market));
            this.SetupEWH();
        }
        unsafe public void SubscribeToTrades(string market, OnTrades onTrades) {
            if (!this.onTradesCbs.ContainsKey(market)) {
                this.onTradesCbs.Add(market, new List<OnTrades>());
            }
            this.onTradesCbs.TryGetValue(market, out var callbacks);
            callbacks.Add(onTrades);
            handleFFIResult(SubscribeToTrades(this._sub_handle, market));
            this.SetupEWH();
        }

        unsafe public void Disconnect() {
            Disconnect(_sub_handle);
        }
    }
}
