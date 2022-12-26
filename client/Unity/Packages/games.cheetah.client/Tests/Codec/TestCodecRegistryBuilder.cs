using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Types.Field;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Codec
{
    public class TestCodecRegistryBuilder
    {
        struct A
        {
        }

        private class CodecA : Codec<A>
        {
            public void Decode(ref NetworkBuffer buffer, ref A dest)
            {
            }

            public void Encode(in A source, ref NetworkBuffer buffer)
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

            public void Decode(ref NetworkBuffer buffer, ref B dest)
            {
            }

            public void Encode(in B source, ref NetworkBuffer buffer)
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