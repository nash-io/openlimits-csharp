use std::{convert::{TryInto}};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use chrono::Duration;
use openlimits::{
  exchange::{OpenLimits, ExchangeAccount, ExchangeMarketData}, 
  exchange_ws::OpenLimitsWs, 
  exchange_info::{MarketPair, ExchangeInfoRetrieval},
  errors::OpenLimitError,
  any_exchange::{AnyExchange, InitAnyExchange, AnyWsExchange},
  nash::{
    NashCredentials,
    NashParameters,
    Environment
  },
  binance::{
    BinanceCredentials,
    BinanceParameters,
  },
  model::{      
    OrderBookRequest, 
    Liquidity,
    Side,
    CancelAllOrdersRequest, 
    CancelOrderRequest,
    OrderType,
    AskBid,
    TimeInForce,
    OpenLimitOrderRequest,
    OrderStatus,
    OpenMarketOrderRequest,
    GetOrderHistoryRequest,
    TradeHistoryRequest,
    GetHistoricTradesRequest,
    GetHistoricRatesRequest,
    GetPriceTickerRequest,
    Paginator,
    Balance,
    Order,
    Trade,
    Interval,
    Candle,
    websocket::{Subscription, OpenLimitsWebSocketMessage, WebSocketResponse}
  }
};
use tokio::stream::StreamExt;
use std::{ffi::CStr, ffi::CString, os::raw::c_char};
use thiserror::Error;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFIMarketPair {
  base: *mut c_char,
  quote: *mut c_char,
  symbol: *mut c_char,
  base_increment: *mut c_char,
  quote_increment: *mut c_char,
  base_min_price: *mut c_char,
  quote_min_price: *mut c_char,
}

fn interval_from_string(
  str: String
) -> Result<Interval, String> {
  match str.as_str() {
    "OneMinute" => Ok(Interval::OneMinute),
    "ThreeMinutes" => Ok(Interval::ThreeMinutes),
    "FiveMinutes" => Ok(Interval::FiveMinutes),
    "FifteenMinutes" => Ok(Interval::FifteenMinutes),
    "ThirtyMinutes" => Ok(Interval::ThirtyMinutes),
    "OneHour" => Ok(Interval::OneHour),
    "TwoHours" => Ok(Interval::TwoHours),
    "FourHours" => Ok(Interval::FourHours),
    "SixHours" => Ok(Interval::SixHours),
    "EightHours" => Ok(Interval::EightHours),
    "TwelveHours" => Ok(Interval::TwelveHours),
    "OneDay" => Ok(Interval::OneDay),
    "ThreeDays" => Ok(Interval::ThreeDays),
    "OneWeek" => Ok(Interval::OneWeek),
    "OneMonth" => Ok(Interval::OneMonth),
    _ => Err(format!("Invalid interval string {}", str))
  }
}


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFICandle {
  time: u64,
  low: f64,
  high: f64,
  open: f64,
  close: f64,
  volume: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFIPaginator {
  start_time: u64,
  end_time: u64,
  limit: u64,
  before: *mut c_char,
  after: *mut c_char,
}


fn string_to_c_str(s: String) -> *mut c_char {
  let cex = CString::new(s).expect("Failed to create CString!");
  let raw = cex.into_raw();
  // println!("Handling ownership of {:?} to c#", raw);

  raw
}


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFIBalance {
  asset: *mut c_char,
  total: *mut c_char,
  free: *mut c_char,
}

fn to_ffi_balance(b: Balance) -> FFIBalance {
  FFIBalance {
    asset: string_to_c_str(b.asset),
    total: string_to_c_str(b.total.to_string()),
    free: string_to_c_str(b.free.to_string())
  }
}

fn market_pair_to_ffi(pair: MarketPair) -> FFIMarketPair {
  let base_min_price = pair.min_base_trade_size.map(|f|string_to_c_str(f.to_string())).unwrap_or(std::ptr::null_mut());
  let quote_min_price = pair.min_quote_trade_size.map(|f|string_to_c_str(f.to_string())).unwrap_or(std::ptr::null_mut());

  FFIMarketPair {
    base: string_to_c_str(pair.base),
    quote: string_to_c_str(pair.quote),
    symbol: string_to_c_str(pair.symbol),
    base_increment: string_to_c_str(pair.base_increment.to_string()),
    quote_increment: string_to_c_str(pair.quote_increment.to_string()),
    base_min_price,
    quote_min_price,
  }
}

fn c_str_to_string(s: *mut c_char) -> Result<String, std::str::Utf8Error> {
  let str = unsafe { CStr::from_ptr(s) };
  str.to_str().map(String::from)
}
fn nullable_cstr(s: *mut c_char) -> Result<Option<String>, std::str::Utf8Error> {
  if s.is_null() {
    Ok(None)
  } else {
    c_str_to_string(s).map(Some)
  }
}


impl TryInto<Paginator> for FFIPaginator {
  type Error = std::str::Utf8Error;
  fn try_into(self) -> Result<Paginator, Self::Error> {
    Ok(
      Paginator {
        start_time: match self.start_time { 0 => None, v => Some(v) },
        end_time: match self.end_time { 0 => None, v => Some(v) },
        limit: match self.limit { 0 => None, v => Some(v) },
        before: nullable_cstr(self.before)?,
        after: nullable_cstr(self.after)?,
      }
    )
  }
}


#[derive(Error, Debug)]
pub enum OpenlimitsSharpError {
  #[error("Invalid argument {0}")]
  InvalidArgument(String),
  #[error("{0}")]
  OpenLimitsError(#[from] OpenLimitError)
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum OpenLimitsResultTag {
  Ok,
  InvalidArgument,
  BinanceError,
  CoinbaseError,
  NashProtocolError,
  MissingImplementation,
  AssetNotFound,
  NoApiKeySet,
  InternalServerError,
  ServiceUnavailable,
  Unauthorized,
  SymbolNotFound,
  SocketError,
  GetTimestampFailed,
  ReqError,
  InvalidHeaderError,
  InvalidPayloadSignature,
  IoError,
  PoisonError,
  JsonError,
  ParseFloatError,
  UrlParserError,
  Tungstenite,
  TimestampError,
  UnkownResponse,
  NotParsableResponse,
  MissingParameter,

  WebSocketMessageNotSupported
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OpenLimitsResult {
  tag: OpenLimitsResultTag,
  message: *mut c_char
}

fn result_to_ffi(r: Result<(), OpenlimitsSharpError>) -> OpenLimitsResult {
  match r {
    Ok(_) => OpenLimitsResult { tag: OpenLimitsResultTag::Ok, message: std::ptr::null_mut() },
    Err(e) => {
      match e {
        OpenlimitsSharpError::InvalidArgument(msg) => OpenLimitsResult { tag: OpenLimitsResultTag::InvalidArgument, message: string_to_c_str(msg) },
        OpenlimitsSharpError::OpenLimitsError(e) => {
          let message = match &e {
            OpenLimitError::BinanceError(e) => e.msg.clone(),
            OpenLimitError::CoinbaseError(e) => e.message.clone(),
            OpenLimitError::NashProtocolError(e) => e.0.to_string(),
            OpenLimitError::MissingImplementation(e) => e.message.clone(),
            OpenLimitError::AssetNotFound() => String::from("AssetNotFound"),
            OpenLimitError::NoApiKeySet() => String::from("NoApiKeySet"),
            OpenLimitError::InternalServerError() => String::from("InternalServerError"),
            OpenLimitError::ServiceUnavailable() => String::from("ServiceUnavailable"),
            OpenLimitError::Unauthorized() => String::from("Unauthorized"),
            OpenLimitError::SymbolNotFound() => String::from("SymbolNotFound"),
            OpenLimitError::SocketError() => String::from("SocketError"),
            OpenLimitError::GetTimestampFailed() => String::from("GetTimestampFailed"),
            OpenLimitError::ReqError(e) => e.to_string(),
            OpenLimitError::InvalidHeaderError(e) => e.to_string(),
            OpenLimitError::InvalidPayloadSignature(e) => e.to_string(),
            OpenLimitError::IoError(e) => e.to_string(),
            OpenLimitError::PoisonError() => String::from("PoisonError"),
            OpenLimitError::JsonError(e) => e.to_string(),
            OpenLimitError::ParseFloatError(e) => e.to_string(),
            OpenLimitError::UrlParserError(e) => e.to_string(),
            OpenLimitError::Tungstenite(e) => e.to_string(),
            OpenLimitError::TimestampError(e) => e.to_string(),
            OpenLimitError::UnkownResponse(e) => e.clone(),
            OpenLimitError::NotParsableResponse(e) => e.clone(),
            OpenLimitError::MissingParameter(e) => e.clone(),
            OpenLimitError::WebSocketMessageNotSupported() => String::from("WebSocketMessageNotSupported"),
          };
          let tag = match &e {
            OpenLimitError::BinanceError(_) => OpenLimitsResultTag::BinanceError,
            OpenLimitError::CoinbaseError(_) => OpenLimitsResultTag::CoinbaseError,
            OpenLimitError::NashProtocolError(_) => OpenLimitsResultTag::NashProtocolError,
            OpenLimitError::MissingImplementation(_) => OpenLimitsResultTag::MissingImplementation,
            OpenLimitError::AssetNotFound() => OpenLimitsResultTag::AssetNotFound,
            OpenLimitError::NoApiKeySet() => OpenLimitsResultTag::NoApiKeySet,
            OpenLimitError::InternalServerError() => OpenLimitsResultTag::InternalServerError,
            OpenLimitError::ServiceUnavailable() => OpenLimitsResultTag::ServiceUnavailable,
            OpenLimitError::Unauthorized() => OpenLimitsResultTag::Unauthorized,
            OpenLimitError::SymbolNotFound() => OpenLimitsResultTag::SymbolNotFound,
            OpenLimitError::SocketError() => OpenLimitsResultTag::SocketError,
            OpenLimitError::GetTimestampFailed() => OpenLimitsResultTag::GetTimestampFailed,
            OpenLimitError::ReqError(_) => OpenLimitsResultTag::ReqError,
            OpenLimitError::InvalidHeaderError(_) => OpenLimitsResultTag::InvalidHeaderError,
            OpenLimitError::InvalidPayloadSignature(_) => OpenLimitsResultTag::InvalidPayloadSignature,
            OpenLimitError::IoError(_) => OpenLimitsResultTag::IoError,
            OpenLimitError::PoisonError() => OpenLimitsResultTag::PoisonError,
            OpenLimitError::JsonError(_) => OpenLimitsResultTag::JsonError,
            OpenLimitError::ParseFloatError(_) => OpenLimitsResultTag::ParseFloatError,
            OpenLimitError::UrlParserError(_) => OpenLimitsResultTag::UrlParserError,
            OpenLimitError::Tungstenite(_) => OpenLimitsResultTag::Tungstenite,
            OpenLimitError::TimestampError(_) => OpenLimitsResultTag::TimestampError,
            OpenLimitError::UnkownResponse(_) => OpenLimitsResultTag::UnkownResponse,
            OpenLimitError::NotParsableResponse(_) => OpenLimitsResultTag::NotParsableResponse,
            OpenLimitError::MissingParameter(_) => OpenLimitsResultTag::MissingParameter,
            OpenLimitError::WebSocketMessageNotSupported() => OpenLimitsResultTag::WebSocketMessageNotSupported
          };
          OpenLimitsResult { tag, message: string_to_c_str(message) }
        },
      }
    }
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFIAskBid {
  pub price: f64,
  pub qty: f64,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum FFILiquidity {
  Unknown,
  Maker,
  Taker,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum FFISide {
  Buy,
  Sell,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum FFITIF {
  GTC,
  FOK,
  IOC,
  GTT
}

fn ffitif_to_tif(tif: FFITIF, ms: u64) -> TimeInForce {
  match tif {
    FFITIF::GTC => TimeInForce::GoodTillCancelled,
    FFITIF::IOC => TimeInForce::ImmediateOrCancelled,
    FFITIF::FOK => TimeInForce::FillOrKill,
    FFITIF::GTT => TimeInForce::GoodTillTime(
      Duration::milliseconds(ms as i64)
    ),
  }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum FFIOrderType {
  Limit,
  Market,
  StopLimit,
  StopMarket,
  Unknown,
}

fn order_type_to_ffi(t: OrderType) -> FFIOrderType {
  match t {
    OrderType::Limit => FFIOrderType::Limit,
    OrderType::Market => FFIOrderType::Market,
    OrderType::StopLimit => FFIOrderType::StopLimit,
    OrderType::StopMarket => FFIOrderType::StopMarket,
    OrderType::Unknown => FFIOrderType::Unknown,
  }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum FFIOrderStatus {
  New,
  PartiallyFilled,
  Filled,
  Canceled,
  PendingCancel,
  Rejected,
  Expired,
  Open,
  Pending,
  Active,
}


fn order_status_to_ffi(t: OrderStatus) -> FFIOrderStatus {
  match t {
    OrderStatus::New => FFIOrderStatus::New,
    OrderStatus::PartiallyFilled => FFIOrderStatus::PartiallyFilled,
    OrderStatus::Filled => FFIOrderStatus::Filled,
    OrderStatus::Canceled => FFIOrderStatus::Canceled,
    OrderStatus::PendingCancel => FFIOrderStatus::PendingCancel,
    OrderStatus::Rejected => FFIOrderStatus::Rejected,
    OrderStatus::Expired => FFIOrderStatus::Expired,
    OrderStatus::Open => FFIOrderStatus::Open,
    OrderStatus::Pending => FFIOrderStatus::Pending,
    OrderStatus::Active => FFIOrderStatus::Active,
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFITrade {
  id: *mut c_char,
  order_id: *mut c_char,
  market_pair: *mut c_char,
  price: f64,
  qty: f64,
  fees: f64,
  side: FFISide,
  liquidity: FFILiquidity,
  created_at: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFIOrder {
  pub id: *mut c_char,
  pub market_pair: *mut c_char,
  pub client_order_id: *mut c_char,
  pub created_at: u64,
  pub order_type: FFIOrderType,
  pub side: FFISide,
  pub status: FFIOrderStatus,
  pub size: f64,
  pub price: f64,
}

fn order_to_ffi(t: Order) -> FFIOrder {
  FFIOrder {
    id: string_to_c_str(t.id),
    market_pair: string_to_c_str(t.market_pair),
    client_order_id: match t.client_order_id {
      None => std::ptr::null_mut(),
      Some(client_order_id) => string_to_c_str(client_order_id)
    },
    created_at: match t.created_at {
      None => 0,
      Some(created_at) => created_at
    },
    order_type: order_type_to_ffi(t.order_type),
    side: match t.side {
      Side::Buy => FFISide::Buy,
      Side::Sell => FFISide::Sell,
    },
    status: order_status_to_ffi(t.status),
    size: t.size.to_f64().unwrap_or_default(),
    price: match t.price {
      Some(price) => price.to_f64().unwrap_or_default(),
      None => std::f64::NAN
    },
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFIGetHistoricTradesRequest {
  market: *mut c_char,
  paginator: *mut FFIPaginator
}


fn to_ffi_ask_bid(f: &AskBid) -> FFIAskBid {
  FFIAskBid {
    price: f.price.to_f64().unwrap(),
    qty: f.qty.to_f64().unwrap()
  }
}

fn to_ffi_candle(f: &Candle) -> FFICandle {
  FFICandle {
    time: f.time,
    low: f.low.to_f64().unwrap(),
    high: f.high.to_f64().unwrap(),
    open: f.open.to_f64().unwrap(),
    close: f.close.to_f64().unwrap(),
    volume: f.volume.to_f64().unwrap(),
  }
}

fn to_ffi_trade(f: &Trade) -> FFITrade {
  FFITrade {
    id: string_to_c_str(f.id.clone()),
    order_id: string_to_c_str(f.order_id.clone()),
    market_pair: string_to_c_str(f.market_pair.clone()),
    price: f.price.to_f64().unwrap_or_default(),
    qty: f.qty.to_f64().unwrap_or_default(),
    fees: match f.fees {
      Some(f) => f.to_f64().unwrap_or_default(),
      None => 0.0,
    },
    side: match f.side {
      Side::Buy => FFISide::Buy,
      Side::Sell => FFISide::Sell,
    },
    liquidity: match f.liquidity {
      Some(Liquidity::Maker) => FFILiquidity::Maker,
      Some(Liquidity::Taker) => FFILiquidity::Taker,
      None => FFILiquidity::Unknown,
    },
    created_at: f.created_at,
  }
}

#[repr(C)]
#[derive(Debug)]
pub struct FFIBinanceConfig {
    apikey: *mut c_char,
    secret: *mut c_char,
    sandbox: bool
}

type Out<T> = *mut T;


fn binance_credentials_from_ptrs(apikey: *mut c_char, secret: *mut c_char) -> Result<Option<BinanceCredentials>, std::str::Utf8Error> {
  if apikey.is_null() {
    return Ok(None)
  }
  if secret.is_null() {
    return Ok(None)
  }

  Ok(
    Some(
      BinanceCredentials {
        api_key: c_str_to_string(apikey)?,
        api_secret: c_str_to_string(secret)?
      }
    )
  )
}

impl TryInto<BinanceParameters> for FFIBinanceConfig {
  type Error = ();
  fn try_into(self) -> Result<BinanceParameters, Self::Error> {
    Ok(
      BinanceParameters {
        credentials: binance_credentials_from_ptrs(self.apikey, self.secret).map_err(|_|())?,
        sandbox: self.sandbox,
      }
    )
  }
}

#[repr(u32)]
#[derive(Debug)]
pub enum FFINashEnv {
  Sandbox,
  Production
}

#[repr(C)]
pub struct ExchangeClient {
  client: AnyExchange,
  init_params: InitAnyExchange,
  channel: Option<tokio::sync::mpsc::UnboundedSender<SubthreadCmd>>,
  runtime: tokio::runtime::Runtime
}

#[repr(C)]
#[derive(Debug)]
pub struct InitResult {
  client: *mut ExchangeClient,
}
pub enum SubthreadCmd {
  Sub(Subscription),
  Disconnect
}

#[no_mangle]
pub  extern "cdecl" fn init_binance(config: FFIBinanceConfig) -> *mut ExchangeClient {
  let init_params: InitAnyExchange = config.try_into().map(InitAnyExchange::Binance).expect("Failed to parse params");
  let mut runtime = tokio::runtime::Builder::new().basic_scheduler().enable_all().build().expect("Failed to create runtime");
  
  let client_future = OpenLimits::instantiate(init_params.clone());
  let client: AnyExchange = runtime.block_on(client_future);


  let b = Box::new(ExchangeClient{
    client,
    init_params,
    channel: None,
    runtime
  });
  Box::into_raw(b)
}

#[no_mangle]
pub  extern "cdecl" fn init_nash(
  apikey: *mut c_char,
  secret: *mut c_char,
  client_id: u64,
  environment: FFINashEnv,
  timeout: u64,
  affiliate_code: *mut c_char,
)  -> *mut ExchangeClient {
  let mut credentials: Option<NashCredentials> = None;
  if !apikey.is_null() && !secret.is_null() {
    credentials = Some(
      NashCredentials {
        secret: c_str_to_string(secret).expect("failed to decode secret"),
        session: c_str_to_string(apikey).expect("failed to decode apikey")
      }
    )
  }

  let environment = match environment {
    FFINashEnv::Production => Environment::Production,
    FFINashEnv::Sandbox => Environment::Sandbox,
  };

  let affiliate_code = nullable_cstr(affiliate_code).unwrap();

  let nash_params =  NashParameters {
    affiliate_code,
    credentials,
    client_id,
    timeout,
    environment
  };

  let init_params = InitAnyExchange::Nash(
    nash_params
  );

  let mut runtime = tokio::runtime::Builder::new().basic_scheduler().enable_all().build().expect("Failed to create runtime");

  let client_future = OpenLimits::instantiate(init_params.clone());
  let client: AnyExchange = runtime.block_on(client_future);

  let b = Box::new(ExchangeClient{
    client,
    init_params,
    channel: None,
    runtime
  });
  Box::into_raw(b)
}

#[no_mangle]
pub  extern "cdecl" fn order_book(
  client: *mut ExchangeClient,
  market: *mut c_char,
  bids_buff: *mut FFIAskBid, bids_buff_len: u64, actual_bids_buff_len: Out<u64>,
  asks_buff: *mut FFIAskBid, asks_buff_len: u64, actual_asks_buff_len: Out<u64>,
  last_update_id: Out<u64>,
  update_id: Out<u64>,
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError>{
    if client.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("client is null")));
    }

    if market.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("market is null")));
    }
    let market_pair = c_str_to_string(market).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse market string. Invalid character on pos {}", e.valid_up_to()))
    )?;
    
    let req = OrderBookRequest {
      market_pair
    };
    unsafe {
      let resp = (*client).runtime.block_on(
        (*client).client.order_book(&req)
      )?;
  
      let bids = std::slice::from_raw_parts_mut::<FFIAskBid>(bids_buff, bids_buff_len as usize);
      let ffi_bids: Vec<FFIAskBid> = resp.bids.iter().map(to_ffi_ask_bid).collect();
      let l = std::cmp::min(bids_buff_len as usize, ffi_bids.len() as usize);
      bids[0..l].copy_from_slice(&ffi_bids[0..l]);
      (*actual_bids_buff_len) = l as u64;
  
      let asks = std::slice::from_raw_parts_mut::<FFIAskBid>(asks_buff, asks_buff_len as usize);
      let ffi_asks: Vec<FFIAskBid> = resp.asks.iter().map(to_ffi_ask_bid).collect();
      let l = std::cmp::min(asks_buff_len as usize, ffi_asks.len() as usize);
      asks[0..l].copy_from_slice(&ffi_asks[0..l]);
      (*actual_asks_buff_len) = l as u64;
      (*last_update_id) = resp.last_update_id.unwrap_or_default();
      (*update_id) = resp.update_id.unwrap_or_default();
    };
    Ok(())
  };

  result_to_ffi(call())
}



#[no_mangle]
pub  extern "cdecl" fn get_price_ticker(
  client: *mut ExchangeClient,
  market: *mut c_char,
  price: Out<f64>
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if client.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("client is null")));
    }

    if market.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("market is null")));
    }
    let market_pair = c_str_to_string(market).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse market string. Invalid character on pos {}", e.valid_up_to()))
    )?;

    let req = GetPriceTickerRequest {
      market_pair
    };
    unsafe {
      let resp = (*client).runtime.block_on(
        (*client).client.get_price_ticker(&req)
      )?;
      let price_opt = resp.price;
      let price_opt = price_opt.map(|f| f.to_f64()).flatten();
      (*price) = price_opt.unwrap_or(std::f64::NAN);
      Ok(())
    }
  };
  

  result_to_ffi(call())
}


#[no_mangle]
pub  extern "cdecl" fn get_historic_rates(
  client: *mut ExchangeClient,
  market: *mut c_char,
  interval: *mut c_char,
  paginator: *mut FFIPaginator,
  candles_buff: *mut FFICandle, candles_buff_len: usize, actual_candles_buff_len: Out<usize>,
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if client.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("client is null")));
    }
    let mut paginator_res: Option<Result<Paginator, _>> = None;
    if !paginator.is_null() {
      unsafe {
        let pag: Result<Paginator, _> = (*paginator).try_into();
        paginator_res = Some(pag);
      }
    }
    let paginator = paginator_res.transpose().map_err(|_| OpenlimitsSharpError::InvalidArgument(String::from("Invalid paginator")))?;
    let market_pair = c_str_to_string(market).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse market string. Invalid character on pos {}", e.valid_up_to()))
    )?;
    let interval = c_str_to_string(interval).map(interval_from_string).map_err(|_| OpenlimitsSharpError::InvalidArgument(String::from("Invalid interval")))?;
    let interval = interval.map_err(|_|OpenlimitsSharpError::InvalidArgument(String::from("Invalid interval")))?;

    let req = GetHistoricRatesRequest {
      paginator,
      market_pair,
      interval
    };
    unsafe {
      let resp = (*client).runtime.block_on(
        (*client).client.get_historic_rates(&req)
      )?;


      let canles = std::slice::from_raw_parts_mut::<FFICandle>(candles_buff, candles_buff_len);
      let ffi_candles: Vec<FFICandle> = resp.iter().map(to_ffi_candle).collect();
      let l = std::cmp::min(candles_buff_len, ffi_candles.len());
      canles[0..l].copy_from_slice(&ffi_candles[0..l]);
      (*actual_candles_buff_len) = l;
      Ok(())
    }
  };
  result_to_ffi(call())
}

#[no_mangle]
pub  extern "cdecl" fn get_historic_trades(
  client: *mut ExchangeClient,
  market: *mut c_char,
  paginator: *mut FFIPaginator,
  buff: *mut FFITrade, buff_len: usize, actual_buff_len: Out<usize>,
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if client.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("client is null")));
    }
    let market_pair = c_str_to_string(market).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse market string. Invalid character on pos {}", e.valid_up_to()))
    )?;
    

    let mut paginator_res: Option<Result<Paginator, _>> = None;
    if !paginator.is_null() {
      unsafe {
        let pag: Result<Paginator, _> = (*paginator).try_into();
        paginator_res = Some(pag);
      }
    }
    let paginator = paginator_res.transpose().map_err(|_| OpenlimitsSharpError::InvalidArgument(String::from("Invalid paginator")))?;

    let req = GetHistoricTradesRequest {
      paginator,
      market_pair,
    };
    unsafe {
      let resp = (*client).runtime.block_on(
        (*client).client.get_historic_trades(&req)
      )?;

      let trades = std::slice::from_raw_parts_mut::<FFITrade>(buff, buff_len);
      let ffi_trades: Vec<FFITrade> = resp.iter().map(to_ffi_trade).collect();
      let l = std::cmp::min(buff_len, ffi_trades.len());
      trades[0..l].copy_from_slice(&ffi_trades[0..l]);
      (*actual_buff_len) = l;
      Ok(())
    }
  };
  result_to_ffi(call())
}
#[no_mangle]

pub extern "cdecl" fn place_order(
  client: *mut ExchangeClient,
  market: *mut c_char,
  qty: *mut c_char,
  limit: bool,
  price: *mut c_char,
  side: FFISide,
  tif: FFITIF,
  tif_duration: u64,
  _post_only: bool,

  result: Out<FFIOrder>
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if client.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("client is null")));
    }
    let market_pair = c_str_to_string(market).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse market string. Invalid character on pos {}", e.valid_up_to()))
    )?;
    let size = c_str_to_string(qty).map(|q| Decimal::from_str(q.as_str()));
    let size = size.map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse size string. Invalid character on pos {}", e.valid_up_to()))
    )?;
    let size = size.map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse size string: {}", e))
    )?;


    if limit == false {
      let  req = OpenMarketOrderRequest {
        market_pair,
        size
      };

      unsafe {
        #[allow(unreachable_patterns)]
        match side {
          FFISide::Buy => {
            let order = (*client).runtime.block_on(
              (*client).client.market_buy(&req)
            )?;
            (*result) = order_to_ffi(order);
            return Ok(());
          },
          FFISide::Sell => {
            let order = (*client).runtime.block_on(
              (*client).client.market_sell(&req)
            )?;
            (*result) = order_to_ffi(order);
            return Ok(());
          },
          e => return Err(OpenlimitsSharpError::InvalidArgument(format!("Invalid side size string: {:?}", e)))
        }
      }
    }
    let price = c_str_to_string(price).map(|q| Decimal::from_str(q.as_str())).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse price string. Invalid character on pos {}", e.valid_up_to()))
    )?;
    let price = price.map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse price string: {}", e))
    )?;

    let time_in_force = ffitif_to_tif(tif, tif_duration);
    let req = OpenLimitOrderRequest {
      market_pair,
      price,
      time_in_force,
      size,
      post_only: _post_only
    };
    unsafe {
      #[allow(unreachable_patterns)]
      match side {
        FFISide::Buy => {
          let order = (*client).runtime.block_on(
            (*client).client.limit_buy(&req)
          )?;
          (*result) = order_to_ffi(order);
          return Ok(());
        },
        FFISide::Sell => {
          let order = (*client).runtime.block_on(
            (*client).client.limit_sell(&req)
          )?;
          (*result) = order_to_ffi(order);
          return Ok(());
        },
        e => return Err(OpenlimitsSharpError::InvalidArgument(format!("Invalid side size string: {:?}", e)))
      }
    }
  };

  result_to_ffi(call())
}

#[no_mangle]
pub  extern "cdecl" fn get_all_open_orders(
  client: *mut ExchangeClient,
  buff: *mut FFIOrder, buff_len: usize, actual_buff_len: Out<usize>,
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if client.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("client is null")));
    }

    unsafe {
      let resp = (*client).runtime.block_on(
        (*client).client.get_all_open_orders()
      )?;

      let orders = std::slice::from_raw_parts_mut::<FFIOrder>(buff, buff_len);
      let ffi_orders: Vec<FFIOrder> = resp.into_iter().map(order_to_ffi).collect();
      let l = std::cmp::min(buff_len, ffi_orders.len());
      orders[0..ffi_orders.len()].copy_from_slice(&ffi_orders[0..l]);
      (*actual_buff_len) = l;
    };
    Ok(())
  };

  result_to_ffi(call())
}

#[no_mangle]
pub  extern "cdecl" fn get_order_history(
  client: *mut ExchangeClient,
  market: *mut c_char,
  paginator: *mut FFIPaginator,
  buff: *mut FFIOrder, buff_len: usize, actual_buff_len: Out<usize>,
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if client.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("client is null")));
    }
    let market_pair = nullable_cstr(market).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse market string. Invalid character on pos {}", e.valid_up_to()))
    )?;
    

    let mut paginator_res: Option<Result<Paginator, _>> = None;
    if !paginator.is_null() {
      unsafe {
        let pag: Result<Paginator, _> = (*paginator).try_into();
        paginator_res = Some(pag);
      }
    }
    let paginator = paginator_res.transpose().map_err(|_| OpenlimitsSharpError::InvalidArgument(String::from("Invalid paginator")))?;

    let req = GetOrderHistoryRequest {
      paginator,
      market_pair,
    };
    unsafe {
      let resp = (*client).runtime.block_on(
        (*client).client.get_order_history(&req)
      )?;

      let orders = std::slice::from_raw_parts_mut::<FFIOrder>(buff, buff_len);
      let ffi_orders: Vec<FFIOrder> = resp.into_iter().map(order_to_ffi).collect();
      let l = std::cmp::min(buff_len, ffi_orders.len());

      orders[0..l].copy_from_slice(&ffi_orders[0..l]);
      (*actual_buff_len) = l;
    }
    Ok(())
  };

  result_to_ffi(call())
}



#[no_mangle]
pub  extern "cdecl" fn get_trade_history(
  client: *mut ExchangeClient,
  market: *mut c_char,
  order_id: *mut c_char,
  paginator: *mut FFIPaginator,
  buff: *mut FFITrade, buff_len: usize, actual_buff_len: Out<usize>,
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if client.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("client is null")));
    }
    let market_pair = nullable_cstr(market).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse market string. Invalid character on pos {}", e.valid_up_to()))
    )?;
    let order_id = nullable_cstr(order_id).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse order_id string. Invalid character on pos {}", e.valid_up_to()))
    )?;


    let mut paginator_res: Option<Result<Paginator, _>> = None;
    if !paginator.is_null() {
      unsafe {
        let pag: Result<Paginator, _> = (*paginator).try_into();
        paginator_res = Some(pag);
      }
    }
    let paginator = paginator_res.transpose().map_err(|_| OpenlimitsSharpError::InvalidArgument(String::from("Invalid paginator")))?;

    let req = TradeHistoryRequest {
      paginator,
      order_id,
      market_pair,
    };
    unsafe {
      let resp = (*client).runtime.block_on(
        (*client).client.get_trade_history(&req)
      )?;

      let trades = std::slice::from_raw_parts_mut::<FFITrade>(buff, buff_len);
      let ffi_trades: Vec<FFITrade> = resp.iter().map(to_ffi_trade).collect();
      let l = std::cmp::min(buff_len, ffi_trades.len());

      trades[0..ffi_trades.len()].copy_from_slice(&ffi_trades[0..l]);
      (*actual_buff_len) = l;
    }
    Ok(())
  };

  result_to_ffi(call())

}


#[no_mangle]
pub  extern "cdecl" fn get_account_balances(
  client: *mut ExchangeClient,
  paginator: *mut FFIPaginator,
  buff: *mut FFIBalance, buff_len: usize, actual_buff_len: Out<usize>,
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if client.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("client is null")));
    }

    let mut paginator_res: Option<Result<Paginator, _>> = None;
    if !paginator.is_null() {
      unsafe {
        let pag: Result<Paginator, _> = (*paginator).try_into();
        paginator_res = Some(pag);
      }
    }
    let paginator = paginator_res.transpose().map_err(|_| OpenlimitsSharpError::InvalidArgument(String::from("Invalid paginator")))?;


    unsafe {
      let resp = (*client).runtime.block_on(
        (*client).client.get_account_balances(paginator)
      )?;

      let balances = std::slice::from_raw_parts_mut::<FFIBalance>(buff, buff_len);
      let ffi_balances: Vec<FFIBalance> = resp.into_iter().map(to_ffi_balance).collect();
      let l = std::cmp::min(buff_len, ffi_balances.len());

      balances[0..l].copy_from_slice(&ffi_balances[0..l]);
      (*actual_buff_len) = l;
    }
    Ok(())
  };
  
  result_to_ffi(call())
}


#[no_mangle]
pub  extern "cdecl" fn cancel_all_orders(
  client: *mut ExchangeClient,
  market: *mut c_char,
  buff: *mut *mut c_char, buff_len: usize, actual_buff_len: Out<usize>,
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if client.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("client is null")));
    }
    let market_pair = nullable_cstr(market).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse market string. Invalid character on pos {}", e.valid_up_to()))
    )?;

  
    unsafe {
      let resp = (*client).runtime.block_on(
        (*client).client.cancel_all_orders(&CancelAllOrdersRequest {
          market_pair
        })
      )?;

      let ids = std::slice::from_raw_parts_mut::<*mut c_char>(buff, buff_len);
      let ffi_ids: Vec<*mut c_char> = resp.into_iter().map(|c|string_to_c_str(c.id)).collect();
      let l = std::cmp::min(buff_len, ffi_ids.len());

      ids[0..l].copy_from_slice(&ffi_ids[0..l]);
      (*actual_buff_len) = l;
    }
    Ok(())
  };  
  result_to_ffi(call())
}

#[no_mangle]
pub  extern "cdecl" fn cancel_order(
  client: *mut ExchangeClient,
  order_id: *mut c_char,
  market: *mut c_char,
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if client.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("client is null")));
    }
    let id = c_str_to_string(order_id).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse market string. Invalid character on pos {}", e.valid_up_to()))
    )?;
    let market_pair = nullable_cstr(market).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse market string. Invalid character on pos {}", e.valid_up_to()))
    )?;

    unsafe {
      (*client).runtime.block_on(
        (*client).client.cancel_order(&CancelOrderRequest {
          id,
          market_pair
        })
      )?;
    }
    Ok(())
  };  
  result_to_ffi(call())
}


#[no_mangle]
pub  extern "cdecl" fn receive_pairs(
  client: *mut ExchangeClient,
  buff: *mut FFIMarketPair, buff_len: usize, actual_buff_len: Out<usize>,
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if client.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("client is null")));
    }
    unsafe {
      let pairs = (*client).runtime.block_on(
        (*client).client.retrieve_pairs()
      )?;

      let pairs_buff = std::slice::from_raw_parts_mut::<FFIMarketPair>(buff, buff_len);
      let pairs_ffi: Vec<FFIMarketPair> = pairs.into_iter().map(market_pair_to_ffi).collect();
      let l = std::cmp::min(buff_len, pairs_ffi.len());

      pairs_buff[0..l].copy_from_slice(&pairs_ffi[0..l]);
      (*actual_buff_len) = l;
    }
    Ok(())
  };
  result_to_ffi(call())
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FFITradeBox(*mut FFITrade);
unsafe impl Send for FFITradeBox {}
unsafe impl Sync for FFITradeBox {}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FFIAskBidBox(*mut FFIAskBid);
unsafe impl Send for FFIAskBidBox {}
unsafe impl Sync for FFIAskBidBox {}

#[no_mangle]
#[allow(unsafe_code)]
pub  extern "cdecl" fn init_subscriptions(
  client: *mut ExchangeClient,
  on_error: extern fn(),
  on_ping: extern fn(),
  on_orderbook: extern fn(bids_len: u64, asks_len: u64, market: *mut c_char, last_update_id: u64, update_id: u64),
  on_trades: extern fn(buff_len: u64, market: *mut c_char),
  on_disconnet: extern fn(),
  bids_buff: FFIAskBidBox, bids_buff_len: usize,
  asks_buff: FFIAskBidBox, asks_buff_len: usize,
  trades_buff: FFITradeBox, trades_buff_len: usize
) -> *mut tokio::sync::mpsc::UnboundedSender<SubthreadCmd> {
  let (sub_request_tx, mut sub_rx) = tokio::sync::mpsc::unbounded_channel::<SubthreadCmd>();

  let init_params = unsafe {
    (*client).init_params.clone()
  };

  
  std::thread::spawn(move || {

    let mut rt = tokio::runtime::Builder::new()
                .basic_scheduler()
                .enable_all()
                .build().expect("Could not create Tokio runtime");
    let client: OpenLimitsWs<AnyWsExchange> = rt.block_on(OpenLimitsWs::instantiate(init_params));
    loop { 
      let subcmd = sub_rx.next();
      // let combined = select(subcmd, msg);

      // let next_msg = rt.block_on(combined);

      let thread_cmd = rt.block_on(subcmd);
      match thread_cmd {
        Some(SubthreadCmd::Disconnect) => {
          break;
        },
        Some(SubthreadCmd::Sub(sub)) => {
          match rt.block_on(client.subscribe(sub.clone(), move |resp| {
            let out_asks = unsafe { std::slice::from_raw_parts_mut::<FFIAskBid>(asks_buff.0, asks_buff_len) };
            let out_bids = unsafe { std::slice::from_raw_parts_mut::<FFIAskBid>(bids_buff.0, bids_buff_len) };
            let resp = match resp {
              Ok(e) => e,
              Err(_) => {
                on_error();
                return
              }
            };
            let resp = match resp {
              WebSocketResponse::Generic(msg) => msg,
              _ => {
                return;
              }
            };

            match resp {
              OpenLimitsWebSocketMessage::Ping => {
                on_ping();
              },
              OpenLimitsWebSocketMessage::Trades(trades) => {
                let out_trades = unsafe { std::slice::from_raw_parts_mut::<FFITrade>(trades_buff.0, trades_buff_len) };
                let market = match sub.clone() {
                  Subscription::Trades(market) => market,
                  _ => panic!("Invalid callback triggered")
                };
                for (i, trade) in trades.iter().enumerate() {
                  out_trades[i] = to_ffi_trade(trade);
                }
                on_trades(trades.len() as u64, string_to_c_str(market));
              },
              OpenLimitsWebSocketMessage::OrderBook(resp) => {
                let market = match sub.clone() {
                  Subscription::OrderBookUpdates(market) => market,
                  _ => panic!("Invalid callback triggered")
                };
                for (i, bid) in resp.bids.iter().enumerate() {
                  out_bids[i] = to_ffi_ask_bid(bid);
                }
                for (i, ask) in resp.asks.iter().enumerate() {
                  out_asks[i] = to_ffi_ask_bid(ask);
                }
                on_orderbook(
                  resp.bids.len() as u64,
                  resp.asks.len() as u64,
                  string_to_c_str(market.clone()),
                  resp.last_update_id.unwrap_or_default(),
                  resp.update_id.unwrap_or_default()
                );
              },
              OpenLimitsWebSocketMessage::OrderBookDiff(resp) => {
                let market = match sub.clone() {
                  Subscription::OrderBookUpdates(market) => market,
                  _ => panic!("Invalid callback triggered")
                };
                for (i, bid) in resp.bids.iter().enumerate() {
                  out_bids[i] = to_ffi_ask_bid(bid);
                }
                for (i, ask) in resp.asks.iter().enumerate() {
                  out_asks[i] = to_ffi_ask_bid(ask);
                }
                on_orderbook(
                  resp.bids.len() as u64,
                  resp.asks.len() as u64,
                  string_to_c_str(market.clone()),
                  resp.last_update_id.unwrap_or_default(),
                  resp.update_id.unwrap_or_default()
                );
              }
            };

          })) {
            Ok(_) => {
              // println!("Subscribed to {:?}", sub);
            },
            Err(msg) => {
              println!("Failed to subscribe: {}", msg);
            }
          };
        },
        None => {}
      }
    }
    on_disconnet();
  });
  Box::into_raw(Box::new(sub_request_tx))
}


#[no_mangle]
pub extern fn free_string(s: *mut c_char) {
    unsafe {
        if s.is_null() { return }
        CString::from_raw(s)
    };
}

#[no_mangle]
pub  extern "cdecl" fn subscribe_orderbook(
  channel: *mut tokio::sync::mpsc::UnboundedSender<SubthreadCmd>,
  market: *mut c_char,
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if channel.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("channel is null")));
    }
    let market_pair = c_str_to_string(market).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse market string. Invalid character on pos {}", e.valid_up_to()))
    )?;

    unsafe {
      let res = (*channel).send(
        SubthreadCmd::Sub(Subscription::OrderBookUpdates(
          market_pair,
        ))
      );
      res.map_err(|_| "Send error").expect("Failed to send message");
    }
    Ok(())
  };
  result_to_ffi(call())
}

#[no_mangle]
pub  extern "cdecl" fn subscribe_trades(
  channel: *mut tokio::sync::mpsc::UnboundedSender<SubthreadCmd>,
  market: *mut c_char
) -> OpenLimitsResult {
  let call = move|| -> Result<(), OpenlimitsSharpError> {
    if channel.is_null() {
      return Err(OpenlimitsSharpError::InvalidArgument(String::from("channel is null")));
    }
    let market_pair = c_str_to_string(market).map_err(|e|
      OpenlimitsSharpError::InvalidArgument(format!("Failed to parse market string. Invalid character on pos {}", e.valid_up_to()))
    )?;

    unsafe {
      let res = (*channel).send(
        SubthreadCmd::Sub(Subscription::Trades(
          market_pair,
        ))
      );
      res.map_err(|_| "Send error").expect("Failed to send message");
    }
    Ok(())
  };
  result_to_ffi(call())
}

#[no_mangle]
pub  extern "cdecl" fn disconnect(
  channel: *mut tokio::sync::mpsc::UnboundedSender<SubthreadCmd>,
) {
  unsafe {
    let res = (*channel).send(
      SubthreadCmd::Disconnect
    );
    res.map_err(|_| "Send error").expect("Failed to send message");
  }
}