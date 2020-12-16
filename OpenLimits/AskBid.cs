namespace OpenLimits
{
    using System;
    using System.Runtime.InteropServices;
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    internal struct FFIAskBid
    {
        public readonly IntPtr price;
        public readonly IntPtr qty;

        public FFIAskBid(IntPtr price, IntPtr qty)
        {
            this.price = price;
            this.qty = qty;
        }

        public void Dispose() {
            ExchangeClient.FreeString(price);
            ExchangeClient.FreeString(qty);
        }

        public AskBid ToAskBid() {
            return new AskBid(
                CString.ToString(this.price),
                CString.ToString(this.qty)
            );
        }
    }

    public struct AskBid
    {
        public readonly decimal price;
        public readonly decimal qty;

        public AskBid(string price, string qty)
        {
            this.price = Decimal.Parse(price);
            this.qty = Decimal.Parse(qty);
        }

        public override string ToString()
        {
            return "AskBid { price=" + price + ", qty=" + qty + "}";
        }
    }
}