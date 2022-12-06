using System.Collections.Generic;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Editor.Generator.Fields.Array.Exceptions;

namespace Games.Cheetah.Client.Editor.Generator.Fields.Array
{
    internal static class Validators
    {
        internal static void ValidateArraySizeField(ArraySizeField arraySizeFieldAttribute, HashSet<string> processedFields,
            HashSet<string> allFields)
        {
            if (processedFields.Contains(arraySizeFieldAttribute.Field)) return;

            if (allFields.Contains(arraySizeFieldAttribute.Field))
            {
                throw new IncorrectOrderArraySizeFieldException(arraySizeFieldAttribute.Field);
            }

            throw new MissingArraySizeFieldException(arraySizeFieldAttribute.Field);
        }
    }
}