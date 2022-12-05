using System;
using Cheetah.Matches.Realtime.Codec.Formatter;

namespace Cheetah.Matches.Realtime.Tests.Codec.Formatter
{
    public class TestDoubleFormatter : AbstractUnmanagedFormatterTest<double, DoubleFormatter>
    {
        protected override double[] GetValues()
        {
            return new[] { Math.PI, Math.E };
        }
    }
}