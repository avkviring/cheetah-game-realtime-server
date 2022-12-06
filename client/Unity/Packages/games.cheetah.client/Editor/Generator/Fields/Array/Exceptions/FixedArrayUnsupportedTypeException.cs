using System;

namespace Games.Cheetah.Client.Editor.Generator.Fields.Array.Exceptions
{
    public class FixedArrayUnsupportedTypeException : Exception
    {
        public FixedArrayUnsupportedTypeException(string elementTypeName) : base("Unsupported type " + elementTypeName + " for fixed array field.")
        {
        }
    }
}