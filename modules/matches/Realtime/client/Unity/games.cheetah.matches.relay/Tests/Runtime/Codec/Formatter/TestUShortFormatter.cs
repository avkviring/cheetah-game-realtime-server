using Cheetah.Matches.Relay.Codec.Formatter;
using Cheetah.Matches.Relay.Types;
using NUnit.Framework;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public class TestUShortFormatter : AbstractUnmanagedFormatterTest<ushort, UShortFormatter>
    {
        [Test]
        [TestCase(ushort.MinValue)]
        [TestCase(ushort.MaxValue)]
        public void Test(ushort value)
        {
            var formatter = UShortFormatter.Instance;
            var buffer = new CheetahBuffer();
            formatter.Write(value, ref buffer);
            Assert.AreEqual(formatter.Read(ref buffer), value);
        }

        protected override ushort[] GetValues()
        {
            return new[] { ushort.MinValue, ushort.MaxValue };
        }
    }
}