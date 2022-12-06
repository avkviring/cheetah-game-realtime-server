using Games_Cheetah_Client_Tests_Codec_Field;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Types;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Codec.Field
{
    public abstract class AbstractFieldTest<T> where T : new()
    {
        [Test]
        public void Test()
        {
            var codecRegistryBuilder = new CodecRegistryBuilder();
            codecRegistryBuilder.Register(_ => new TestCodecArrayFieldInnerCodec());
            codecRegistryBuilder.Register(factory => new TestCodecArrayFieldStructureCodec(factory));

            codecRegistryBuilder.Register(_ => new TestCodecFieldInnerCodec());
            codecRegistryBuilder.Register(factory => new TestCodecFieldStructureCodec(factory));
            codecRegistryBuilder.Register(_ => new TestEnumFieldStructureCodec());
            codecRegistryBuilder.Register(_ => new TestFixedArrayFieldStructureCodec());
            codecRegistryBuilder.Register(_ => new TestFormattedFieldStructureCodec());
            codecRegistryBuilder.Register(_ => new TestFormatterReferencedArrayFieldStructureCodec());
            codecRegistryBuilder.Register(_ => new TestVariableSizeFieldStructureCodec());


            var codecRegistry = codecRegistryBuilder.Build();
            var codec = codecRegistry.GetCodec<T>();
            var source = GetSource();
            var buffer = new CheetahBuffer();
            codec.Encode(in source, ref buffer);
            var result = GetResult();
            buffer.pos = 0;
            codec.Decode(ref buffer, ref result);
            CheckResult(source, result);
        }

        protected abstract void CheckResult(T source, T result);


        protected virtual T GetResult()
        {
            return new T();
        }


        protected abstract T GetSource();
    }
}