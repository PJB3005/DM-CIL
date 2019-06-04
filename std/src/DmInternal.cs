namespace DM
{
    public static class DmInternal
    {
        public static bool Truthy(object x)
        {
            if (x is double d)
            {
                return d != 0f;
            }

            if (x is float f)
            {
                return f != 0f;
            }

            if (x is int i)
            {
                return i != 0f;
            }

            if (x is string s)
            {
                return !string.IsNullOrEmpty(s);
            }

            return x != null;
        }
    }
}