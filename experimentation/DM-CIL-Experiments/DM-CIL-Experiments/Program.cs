using System;

namespace DM_CIL_Experiments
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("Hello World!");
        }

        public virtual void A()
        {
            throw new NotImplementedException();
        }
    }

    class Arogram : Program
    {
        public override void A()
        {
            base.A();

            Console.WriteLine("yes");
        }
    }
}
