namespace OpenLimits
{
    using System.Runtime.InteropServices;
    using System;
    using System.Text;


    [StructLayout(LayoutKind.Sequential)]
    internal struct FFITrade {
        public readonly IntPtr id;
        public readonly IntPtr orderId;
        public readonly IntPtr marketPair;

        public readonly double price;
        public readonly double qty;
        public readonly double fees;
        public readonly Side side;
        public readonly Liquidity liquidity;
        public readonly ulong createdAt;

        public void Dispose() {
            ExchangeClient.FreeString(id);
            ExchangeClient.FreeString(orderId);
            ExchangeClient.FreeString(marketPair);
        }

        public Trade ToTrade() {
            return new Trade(
                CString.ToString(this.id),
                CString.ToString(this.orderId),
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
        public readonly string orderId;
        public readonly string marketPair;
        public readonly double price;
        public readonly double qty;
        public readonly double fees;
        public readonly Side side;
        public readonly Liquidity liquidity;
        public readonly ulong createdAt;

        public Trade(string id, string orderId, string marketPair, double price, double qty, double fees, Side side, Liquidity liquidity, ulong createdAt)
        {
            this.id = id;
            this.orderId = orderId;
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
                ", order_id='" + orderId + '\'' +
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