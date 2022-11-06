using Cheetah.Matches.Realtime.Codec;
using NUnit.Framework;

namespace Cheetah.Matches.Realtime.Tests.Codec.Field
{
    public class TestFixedArrayField : AbstractFieldTest<TestFixedArrayField.Structure>
    {
        [GenerateCodec]
        public struct Structure
        {
            public byte size;
            [ArraySizeField(nameof(size))] public unsafe fixed byte byteArray[3];

            public unsafe fixed byte arrayWithoutSizeVariable[2];

            [ArraySizeField(nameof(size))] [VariableSizeCodec]
            public unsafe fixed uint variableSizeArray[2];
        }

        protected override Structure GetSource()
        {
            unsafe
            {
                var fixedArrayTestStructure = new Structure()
                {
                    size = 2,
                };
                fixedArrayTestStructure.byteArray[0] = 10;
                fixedArrayTestStructure.byteArray[1] = 11;

                fixedArrayTestStructure.arrayWithoutSizeVariable[0] = 30;
                fixedArrayTestStructure.arrayWithoutSizeVariable[1] = 31;

                fixedArrayTestStructure.variableSizeArray[0] = 50;
                fixedArrayTestStructure.variableSizeArray[1] = 51;

                return fixedArrayTestStructure;
            }
        }

        protected override unsafe void CheckResult(Structure source, Structure result)
        {
            Assert.AreEqual(source.size, result.size);
            for (var i = 0; i < source.size; i++)
            {
                Assert.AreEqual(source.byteArray[i], result.byteArray[i]);
                Assert.AreEqual(source.arrayWithoutSizeVariable[i], result.arrayWithoutSizeVariable[i]);
                Assert.AreEqual(source.variableSizeArray[i], result.variableSizeArray[i]);
            }
        }
    }
}