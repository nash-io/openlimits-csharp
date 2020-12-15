namespace OpenLimits
{
    using System;
    internal struct FFIOrder
    {
        public readonly IntPtr id;
        public readonly IntPtr marketPair;
        public readonly IntPtr clientOrderId;
        public readonly ulong createdAt;
        public readonly OrderType orderType;
        public readonly Side side;
        public readonly OrderStatus status;
        public readonly double size;
        public readonly double price;
        public readonly double remaining;

        public void Dispose() {
            ExchangeClient.FreeString(id);
            ExchangeClient.FreeString(marketPair);
            ExchangeClient.FreeString(clientOrderId);
        }

        public Order ToOrder() {
            return new Order(
                CString.ToString(this.id),
                CString.ToString(this.marketPair),
                CString.ToString(this.clientOrderId),
                this.createdAt,
                this.orderType,
                this.side,
                this.status,
                this.size,
                this.price,
                this.remaining
            );
        }
    }

    public struct Order
    {
        public readonly string id;
        public readonly string marketPair;
        public readonly string clientOrderId;
        public readonly ulong createdAt;
        public readonly OrderType orderType;
        public readonly Side side;
        public readonly OrderStatus status;
        public readonly double size;
        public readonly double price;
        public readonly double remaining;

        public Order(string id, string marketPair, string clientOrderId, ulong createdAt, OrderType orderType, Side side, OrderStatus status, double size, double price, double remaining)
        {
            this.id = id;
            this.marketPair = marketPair;
            this.clientOrderId = clientOrderId;
            this.createdAt = createdAt;
            this.orderType = orderType;
            this.side = side;
            this.status = status;
            this.size = size;
            this.price = price;
            this.remaining = remaining;
        }

        public override bool Equals(object obj)
        {
            return base.Equals(obj);
        }

        public override int GetHashCode()
        {
            return base.GetHashCode();
        }

        public override string ToString()
        {
            return "Order{" +
                "id='" + id + '\'' +
                ", market='" + marketPair + '\'' +
                ", clientOrderId='" + clientOrderId + '\'' +
                ", createdAt=" + createdAt +
                ", orderType='" + orderType + '\'' +
                ", side='" + side + '\'' +
                ", status='" + status + '\'' +
                ", size='" + size + '\'' +
                ", price='" + price + '\'' +
                ", remaining='" + remaining + '\'' +
                '}';
        }
    }
}