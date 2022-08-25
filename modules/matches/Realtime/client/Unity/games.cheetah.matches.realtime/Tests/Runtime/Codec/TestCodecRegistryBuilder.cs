using Cheetah.Matches.Realtime.Codec;
using Cheetah.Matches.Realtime.Types;
using NUnit.Framework;

namespace Cheetah.Matches.Realtime.Tests.Codec
{
    public class TestCodecRegistryBuilder
    {
        struct A
        {
        }

        private class CodecA : Codec<A>
        {
            public void Decode(ref CheetahBuffer buffer, ref A dest)
            {
            }

            public void Encode(ref A source, ref CheetahBuffer buffer)
            {
            }
        }

        struct B
        {
            private A a;
        }

        private class CodecB : Codec<B>
        {
            public CodecB(CodecRegistry codecRegistry)
            {
                codecRegistry.GetCodec<A>();
            }

            public void Decode(ref CheetahBuffer buffer, ref B dest)
            {
            }

            public void Encode(ref B source, ref CheetahBuffer buffer)
            {
            }
        }

        [Test]
        public void ShouldResolveDependencies()
        {
            var builder = new CodecRegistryBuilder();
            builder.Register(factory => new CodecB(factory));
            builder.Register(_ => new CodecA());
            var codecRegistry = builder.Build();
            Assert.AreEqual(codecRegistry.GetCodec<A>().GetType(), typeof(CodecA));
            Assert.AreEqual(codecRegistry.GetCodec<B>().GetType(), typeof(CodecB));
        }

        [Test]
        public void ShouldExceptionWhenWrongDependency()
        {
            var builder = new CodecRegistryBuilder();
            builder.Register(factory => new CodecB(factory));
            Assert.Throws<CodecNotFoundException>(() => builder.Build());
        }
    }
}