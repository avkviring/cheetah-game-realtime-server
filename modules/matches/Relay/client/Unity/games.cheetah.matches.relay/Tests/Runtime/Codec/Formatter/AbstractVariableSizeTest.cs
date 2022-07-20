using System.Linq;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Types;
using NUnit.Framework;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public abstract class AbstractVariableSizeFormatterTest<T, F>
        where F : Formatter<T>, ArrayFormatter<T>, new()
    {
        protected abstract T[] GetValues();
        protected abstract int[] GetSizes();

        [Test]
        public void Test()
        {
            var formatter = new F();
            var values = GetValues();
            var sizes = GetSizes();

            for (var index = 0; index < values.Length; index++)
            {
                var value = values[index];
                var size = sizes[index];
                var buffer = new CheetahBuffer();
                formatter.Write(value, ref buffer);
                Assert.AreEqual(buffer.size, size);
                Assert.AreEqual(formatter.Read(ref buffer), value);
            }
        }

        [Test]
        public void TestArray()
        {
            var formatter = new F();
            var source = GetValues();
            var buffer = new CheetahBuffer();
            formatter.WriteArray(source, (uint)source.Length, 0, ref buffer);
            var read = new T[source.Length];
            formatter.ReadArray(ref buffer, read, (uint)source.Length, 0);
            Assert.True(read.SequenceEqual(source));
        }
    }
}