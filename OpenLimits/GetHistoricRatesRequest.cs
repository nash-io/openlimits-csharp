namespace OpenLimits
{
    using System.Runtime.InteropServices;
    [StructLayout(LayoutKind.Sequential)]
    public struct GetHistoricRatesRequest
    {
        public readonly string market;
        public readonly string interval;
        public readonly Paginator paginator;

        public GetHistoricRatesRequest(string market, string interval, Paginator paginator)
        {
            this.market = market;
            this.interval = interval;
            this.paginator = paginator;
        }

        public GetHistoricRatesRequest(string market, string interval)
        {
            this.market = market;
            this.interval = interval;
            this.paginator = null;
        }
    }
}