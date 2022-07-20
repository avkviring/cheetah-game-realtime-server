using System.Linq;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Types;
using NUnit.Framework;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
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
                var buffer = new CheetahBuffer();
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