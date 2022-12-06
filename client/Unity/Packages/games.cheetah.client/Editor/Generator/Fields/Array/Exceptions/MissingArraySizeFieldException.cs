using System;
using Games.Cheetah.Client.Codec;

namespace Games.Cheetah.Client.Editor.Generator.Fields.Array.Exceptions
{
    public class MissingArraySizeFieldException : Exception
    {
        public MissingArraySizeFieldException(string field) : base($"Field {field} set in {nameof(ArraySizeField)} but not present in structure.")
        {
        }
    }
}