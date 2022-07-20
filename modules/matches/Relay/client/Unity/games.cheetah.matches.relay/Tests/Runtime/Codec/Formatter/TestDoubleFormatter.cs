using System;
using Cheetah.Matches.Relay.Codec.Formatter;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public class TestDoubleFormatter : AbstractUnmanagedFormatterTest<double, DoubleFormatter>
    {
        protected override double[] GetValues()
        {
            return new[] { Math.PI, Math.E };
        }
    }
}