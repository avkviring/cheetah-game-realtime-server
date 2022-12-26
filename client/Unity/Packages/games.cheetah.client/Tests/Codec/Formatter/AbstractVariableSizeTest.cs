using System.Linq;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
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
                var buffer = new NetworkBuffer();
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
            var buffer = new NetworkBuffer();
            formatter.WriteArray(source, (uint)source.Length, 0, ref buffer);
            var read = new T[source.Length];
            formatter.ReadArray(ref buffer, read, (uint)source.Length, 0);
            Assert.True(read.SequenceEqual(source));
        }
    }
}