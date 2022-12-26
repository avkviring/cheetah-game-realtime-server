using System.Linq;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public abstract class AbstractUnmanagedFormatterTest<T, F> : AbstractFormatterTest<T, F>
        where T : unmanaged
        where F : Formatter<T>, ArrayFormatter<T>, FixedArrayFormatter<T>, new()
    {
        [Test]
        public void TestFixedArray()
        {
            unsafe
            {
                var formatter = new F();
                var source = GetValues();
                var buffer = new NetworkBuffer();
                fixed (T* sourceBuffer = source)
                {
                    formatter.WriteFixedArray(sourceBuffer, (uint)source.Length, 0, ref buffer);
                }

                var read = new T[source.Length];
                fixed (T* readBuffer = read)
                {
                    formatter.ReadFixedArray(ref buffer, readBuffer, (uint)source.Length, 0);
                }

                Assert.True(read.SequenceEqual(source));
            }
        }
    }
}