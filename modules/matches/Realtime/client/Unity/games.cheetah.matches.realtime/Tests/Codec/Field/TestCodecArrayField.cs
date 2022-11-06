using Cheetah.Matches.Realtime.Codec;
using NUnit.Framework;

namespace Cheetah.Matches.Realtime.Tests.Codec.Field
{
    public class TestCodecArrayField : AbstractFieldTest<TestCodecArrayField.Structure>
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
            [ArraySizeField(nameof(size))] public Inner[] values;
        }

        protected override Structure GetSource()
        {
            return new()
            {
                size = 2,
                values = new Inner[]
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
                values = new Inner[5]
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