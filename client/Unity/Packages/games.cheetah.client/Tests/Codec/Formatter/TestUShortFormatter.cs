using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestUShortFormatter : AbstractUnmanagedFormatterTest<ushort, UShortFormatter>
    {
        [Test]
        [TestCase(ushort.MinValue)]
        [TestCase(ushort.MaxValue)]
        public void Test(ushort value)
        {
            var formatter = UShortFormatter.Instance;
            var buffer = new NetworkBuffer();
            formatter.Write(value, ref buffer);
            Assert.AreEqual(formatter.Read(ref buffer), value);
        }

        protected override ushort[] GetValues()
        {
            return new[] { ushort.MinValue, ushort.MaxValue };
        }
    }
}