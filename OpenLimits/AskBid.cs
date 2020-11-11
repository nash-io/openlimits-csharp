namespace OpenLimits
{
    using System.Runtime.InteropServices;
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public struct AskBid
    {
        public readonly double price;
        public readonly double qty;

        public AskBid(double price, double qty)
        {
            this.price = price;
            this.qty = qty;
        }

        public override string ToString()
        {
            return "AskBid { price=" + price + ", qty=" + qty + "}";
        }
    }
}