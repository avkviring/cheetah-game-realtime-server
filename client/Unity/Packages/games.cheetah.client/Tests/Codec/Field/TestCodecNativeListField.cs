using Games.Cheetah.Client.Codec;
using NUnit.Framework;
using Unity.Collections;

namespace Games.Cheetah.Client.Tests.Codec.Field
{
    public class TestCodecNativeListField : AbstractFieldTest<TestCodecNativeListField.Structure>
    {
        [GenerateCodec]
        public struct Inner
        {
            public int value;
        }

        [GenerateCodec]
        public struct Structure
        {
            public byte size;
            public NativeList<Inner> values;
        }

        protected override Structure GetSource()
        {
            return new()
            {
                size = 2,
                values = new NativeList<Inner>
                {
                    new()
                    {
                        value = 100
                    },
                    new()
                    {
                        value = 105
                    },
                }
            };
        }

        protected override Structure GetResult()
        {
            return new()
            {
                values = new NativeList<Inner>()
            };
        }

        protected override void CheckResult(Structure source, Structure result)
        {
            Assert.AreEqual(source.size, result.size);
            for (var i = 0; i < source.size; i++)
            {
                Assert.AreEqual(source.values[i].value, result.values[i].value);
            }
        }
    }
}