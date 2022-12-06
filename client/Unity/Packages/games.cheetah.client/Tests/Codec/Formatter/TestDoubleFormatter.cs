using System;
using Games.Cheetah.Client.Codec.Formatter;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestDoubleFormatter : AbstractUnmanagedFormatterTest<double, DoubleFormatter>
    {
        protected override double[] GetValues()
        {
            return new[] { Math.PI, Math.E };
        }
    }
}