using System.Linq;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Types;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public abstract class AbstractFormatterTest<T, F>
        where F : Formatter<T>, ArrayFormatter<T>, new()
    {
        protected abstract T[] GetValues();

        [Test]
        public void Test()
        {
            var formatter = new F();
            var values = GetValues();
            foreach (var value in values)
            {
                var buffer = new CheetahBuffer();
                formatter.Write(value, ref buffer);
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