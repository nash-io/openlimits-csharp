namespace OpenLimits
{
    using System.Runtime.InteropServices;
    using System;
    using System.Text;


    [StructLayout(LayoutKind.Sequential)]
    internal struct FFITrade {
        public readonly IntPtr id;
        public readonly IntPtr buyerOrderId;
        public readonly IntPtr sellerOrderId;
        public readonly IntPtr marketPair;

        public readonly double price;
        public readonly double qty;
        public readonly double fees;
        public readonly Side side;
        public readonly Liquidity liquidity;
        public readonly ulong createdAt;

        public void Dispose() {
            ExchangeClient.FreeString(id);
            ExchangeClient.FreeString(buyerOrderId);
            ExchangeClient.FreeString(sellerOrderId);
            ExchangeClient.FreeString(marketPair);
        }

        public Trade ToTrade() {
            return new Trade(
                CString.ToString(this.id),
                CString.ToString(this.buyerOrderId),
                CString.ToString(this.sellerOrderId),
                CString.ToString(this.marketPair),
                this.price,
                this.qty,
                this.fees,
                this.side,
                this.liquidity,
                this.createdAt
            );
        }
    }

    public struct Trade {
        public readonly string id;
        public readonly string buyerOrderId;
        public readonly string sellerOrderId;
        public readonly string marketPair;
        public readonly double price;
        public readonly double qty;
        public readonly double fees;
        public readonly Side side;
        public readonly Liquidity liquidity;
        public readonly ulong createdAt;

        public Trade(string id, string buyerOrderId, string sellerOrderId, string marketPair, double price, double qty, double fees, Side side, Liquidity liquidity, ulong createdAt)
        {
            this.id = id;
            this.buyerOrderId = buyerOrderId;
            this.sellerOrderId = sellerOrderId;
            this.marketPair = marketPair;
            this.price = price;
            this.qty = qty;
            this.fees = fees;
            this.side = side;
            this.liquidity = liquidity;
            this.createdAt = createdAt;
        }

        public override string ToString()
        {
            return "Trade{" +
                "id='" + id + '\'' +
                ", buyer_order_id='" + buyerOrderId + '\'' +
                ", seller_order_id='" + sellerOrderId + '\'' +
                ", market_pair='" + marketPair + '\'' +
                ", price=" + price +
                ", qty=" + qty +
                ", fees=" + fees +
                ", side='" + side + '\'' +
                ", liquidity='" + liquidity + '\'' +
                ", createdAt=" + createdAt +
                '}';
        }
    }
}